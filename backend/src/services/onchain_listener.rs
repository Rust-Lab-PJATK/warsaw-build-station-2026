use crate::data::job::JobStatus;
use crate::services::job::JobService;
use base64::Engine;
use borsh::BorshDeserialize;
use mongodb::Database;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSignaturesForAddressConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};

const RESULT_APPROVED_PREFIX: &str = "Program log: ResultApproved:";
const PROGRAM_DATA_PREFIX: &str = "Program data: ";

pub struct OnchainListener {
    rpc: RpcClient,
    program_id: Pubkey,
    job_service: JobService,
    last_seen_signature: Option<String>,
}

impl OnchainListener {
    pub fn new(rpc_url: &str, program_id: &str, db: Database) -> Result<Self, String> {
        let program_id = Pubkey::from_str(program_id).map_err(|e| e.to_string())?;
        let rpc = RpcClient::new_with_timeout_and_commitment(
            rpc_url.to_string(),
            Duration::from_secs(20),
            CommitmentConfig::confirmed(),
        );
        Ok(Self {
            rpc,
            program_id,
            job_service: JobService::new(db),
            last_seen_signature: None,
        })
    }

    pub async fn tick(&mut self)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let signatures = self.rpc.get_signatures_for_address_with_config(
            &self.program_id,
            RpcSignaturesForAddressConfig {
                limit: Some(50),
                ..RpcSignaturesForAddressConfig::default()
            },
        )?;

        let mut newest: Option<String> = None;

        for signature in signatures {
            let sig_str = signature.signature.to_string();
            if newest.is_none() {
                newest = Some(sig_str.clone());
            }

            if self
                .last_seen_signature
                .as_ref()
                .is_some_and(|last| last == &sig_str)
            {
                break;
            }

            let sig = match Signature::from_str(&sig_str) {
                Ok(sig) => sig,
                Err(_) => continue,
            };
            let tx = self.rpc.get_transaction(&sig, UiTransactionEncoding::JsonParsed);
            let Ok(tx) = tx else { continue };
            if let Some(task_pubkey) = extract_result_approved_task(&tx) {
                self.handle_result_approved(&task_pubkey).await?;
            }
        }

        if newest.is_some() {
            self.last_seen_signature = newest;
        }

        Ok(())
    }

    async fn handle_result_approved(
        &self,
        task_pubkey: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(mut job) = self.job_service.get_by_task_pubkey(task_pubkey).await? else {
            warn!("No job mapped to task_pubkey: {}", task_pubkey);
            return Ok(());
        };

        if job.status == JobStatus::Accepted {
            debug!("Job already accepted for task_pubkey: {}", task_pubkey);
            return Ok(());
        }

        job.transition_to(JobStatus::Accepted);
        self.job_service
            .update_by_task_pubkey(task_pubkey, &job)
            .await?;
        info!("Job marked accepted for task_pubkey: {}", task_pubkey);
        Ok(())
    }
}

fn extract_result_approved_task(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Option<String> {
    let meta = tx.transaction.meta.as_ref()?;
    let log_messages = meta.log_messages.as_ref()?;
    for log in log_messages {
        if let Some(rest) = log.strip_prefix(PROGRAM_DATA_PREFIX) {
            if let Some(task_pubkey) = parse_event_task_from_base64(rest.trim()) {
                return Some(task_pubkey);
            }
        }
        if let Some(rest) = log.strip_prefix(RESULT_APPROVED_PREFIX) {
            let trimmed = rest.trim();
            return parse_task_from_anchor_log(trimmed);
        }
    }
    None
}

fn parse_task_from_anchor_log(log: &str) -> Option<String> {
    let marker = "task: ";
    let start = log.find(marker)? + marker.len();
    let remaining = &log[start..];
    let end = remaining.find(',').unwrap_or(remaining.len());
    Some(remaining[..end].trim().to_string())
}

fn parse_event_task_from_base64(encoded: &str) -> Option<String> {
    let data = base64::engine::general_purpose::STANDARD.decode(encoded).ok()?;
    if data.len() < 8 {
        return None;
    }

    let (disc, payload) = data.split_at(8);
    if disc != result_approved_discriminator().as_slice() {
        return None;
    }

    let event = ResultApprovedEvent::try_from_slice(payload).ok()?;
    Some(event.task.to_string())
}

fn result_approved_discriminator() -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"event:ResultApproved");
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

#[derive(BorshDeserialize, Debug)]
struct ResultApprovedEvent {
    pub task: Pubkey,
    pub agent: Pubkey,
    pub advance_paid: u64,
    pub remaining_reward: u64,
}

#[cfg(test)]
mod tests {
    use super::{parse_event_task_from_base64, parse_task_from_anchor_log};
    use base64::Engine;
    use borsh::BorshSerialize;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn parse_task_pubkey_from_anchor_log() {
        let log = "task: 7fYf9W4q8ToY6FzbkD8PQ9WCG3GQ6x1Zr2M5y8w3M1hJ, agent: 3k4Y";
        assert_eq!(
            parse_task_from_anchor_log(log),
            Some("7fYf9W4q8ToY6FzbkD8PQ9WCG3GQ6x1Zr2M5y8w3M1hJ".to_string())
        );
    }

    #[test]
    fn parse_event_from_base64() {
        let task = Pubkey::new_unique();
        let agent = Pubkey::new_unique();
        let event = super::ResultApprovedEvent {
            task,
            agent,
            advance_paid: 10,
            remaining_reward: 20,
        };

        let mut data = Vec::new();
        data.extend_from_slice(&super::result_approved_discriminator());
        data.extend_from_slice(&event.try_to_vec().expect("serialize"));
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);

        assert_eq!(parse_event_task_from_base64(&encoded), Some(task.to_string()));
    }
}
