#[cfg(test)]
mod job_history_tests {
    use backend::data::job::{JobHistoryEntry, JobTask};

    #[test]
    fn create_job_history_with_valid_data() {
        let tasks = vec![JobTask {
            title: "Analysis".to_string(),
            description: "Analyze requirements".to_string(),
            price_sol: 100.0,
            complexity: 2,
            rationale: "Essential step".to_string(),
        }];

        let entry = JobHistoryEntry::new(1, tasks, 100.0, 2, "Project analysis".to_string());

        assert_eq!(entry.job_id, 1);
        assert_eq!(entry.total_price_sol, 100.0);
        assert_eq!(entry.overall_complexity, 2);
        assert!(entry.id.is_some());
        assert!(entry.created_at.is_some());
        assert!(entry.updated_at.is_some());
    }

    #[test]
    fn job_history_has_consistent_timestamps() {
        let tasks = vec![];
        let entry = JobHistoryEntry::new(1, tasks, 100.0, 2, "Test".to_string());

        // created_at and updated_at should be equal for new records
        assert_eq!(entry.created_at, entry.updated_at);
    }

    #[test]
    fn job_history_with_multiple_tasks() {
        let tasks = vec![
            JobTask {
                title: "Task 1".to_string(),
                description: "Description 1".to_string(),
                price_sol: 200.0,
                complexity: 3,
                rationale: "Reason 1".to_string(),
            },
            JobTask {
                title: "Task 2".to_string(),
                description: "Description 2".to_string(),
                price_sol: 150.0,
                complexity: 2,
                rationale: "Reason 2".to_string(),
            },
        ];

        let job_history =
            JobHistoryEntry::new(2, tasks.clone(), 350.0, 5, "Multi-task project".to_string());

        assert_eq!(job_history.job_id, 2);
        assert_eq!(job_history.tasks.len(), 2);
        assert_eq!(job_history.total_price_sol, 350.0);
        assert_eq!(job_history.overall_complexity, 5);
    }
}
