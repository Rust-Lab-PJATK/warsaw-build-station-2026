# agents.md — UI Build Spec: Decentralized AI Agent Task Marketplace

> Dokument źródłowy dla agenta AI budującego frontend.
> Traktuj go jako prawdę absolutną przy każdej decyzji projektowej i implementacyjnej.
> Nie odchodź od tych wytycznych bez wyraźnego polecenia użytkownika.

---

## 0. Kontekst Projektu (czytaj przed każdą sesją)

Budujesz **dwustronny rynek zadań** dla wyspecjalizowanych agentów AI osadzony na blockchainie Solana.

**Dwie persony:**
| Persona | Co robi | Główne akcje |
|---|---|---|
| **Klient** | Zleca zadania, finansuje Escrow, zatwierdza wyniki | Post Task → Fund Escrow → Approve / Dispute |
| **Agent AI** | Przegląda zadania, claimuje zlecenie, dostarcza dowód pracy | Browse → Claim → Submit Proof → Receive Payment |

**Kluczowy state machine on-chain:**
```
OPEN → CLAIMED → IN_PROGRESS → PREVIEW_SUBMITTED → APPROVED → PAID
                                                  ↘ DISPUTED
```

**Priorytet demo (hackathon):** Jeden *killer screen* — strona szczegółów zadania — który w 3 minuty pokazuje działanie całego systemu. Głębokość > szerokość.

---

## 1. Filozofia Designu

### Estetyka: **Dark Utilitarian Precision**

Inspiracje: Linear.app, Vercel Dashboard, Bloomberg Terminal, Raycast.
Słowa klucze: precyzja, gęstość informacji, zero dekoracji bez funkcji.

**ZAKAZY — nigdy nie używaj:**
- Purpury, różu, fioletu w jakiejkolwiek formie
- Gradientów tęczowych lub "holograficznych"
- Zaokrągleń > `8px` (poza modalami: max `12px`)
- Cieni box-shadow większych niż subtelny `0 1px 3px`
- Fontów: Inter, Roboto, Arial, system-ui jako głównego fonta UI
- Animacji trwających > 300ms (wyjątek: Escrow progress bar)
- Więcej niż jednego CTA pierwszoplanowego na ekranie

### Zasady UX:
1. **Stan jest zawsze widoczny** — użytkownik nigdy nie zastanawia się "co teraz?"
2. **Dane on-chain jako design element** — hash tx, adresy portfeli, salda to ozdobniki, nie szum
3. **Jedno CTA naraz** — tylko jedna akcja `primary` na widok; reszta `ghost` lub `outline`
4. **Animacje komunikują stan** — każda animacja sygnalizuje zmianę, nie jest dekoracją
5. **Density over sprawl** — informacja gęsta, ale oddychająca; brak pustych ekranów

---

## 2. System Kolorów (CSS Variables — obowiązkowe)

Zdefiniuj w `src/styles/globals.css`. Używaj WYŁĄCZNIE tych zmiennych w kodzie.

```css
:root {
  /* === Tła === */
  --bg-base:        #09090B;   /* główne tło aplikacji (zinc-950) */
  --bg-surface:     #111113;   /* karty, panele */
  --bg-elevated:    #1C1C1F;   /* hover stany, dropdowny */
  --bg-overlay:     #27272A;   /* modalne tła, tooltips */

  /* === Bordering === */
  --border-subtle:  #27272A;   /* domyślne obramowania (zinc-800) */
  --border-default: #3F3F46;   /* aktywne obramowania (zinc-700) */
  --border-strong:  #52525B;   /* focus ringi */

  /* === Tekst === */
  --text-primary:   #FAFAFA;   /* nagłówki, ważne dane */
  --text-secondary: #A1A1AA;   /* opisy, metadane (zinc-400) */
  --text-muted:     #52525B;   /* placeholdery, disabled (zinc-600) */
  --text-inverse:   #09090B;   /* tekst na jasnych buttonach */

  /* === Akcenty — JEDYNE dozwolone kolory akcentu === */
  --accent-primary:       #00E5A0;   /* główny zielony akcent (CTA, success) */
  --accent-primary-dim:   rgba(0, 229, 160, 0.08);
  --accent-primary-hover: #00C98C;   /* hover na primary button */

  --accent-amber:     #F59E0B;   /* ostrzeżenia, "w toku", countdown < 1h */
  --accent-amber-dim: rgba(245, 158, 11, 0.08);
  --accent-red:       #EF4444;   /* błędy, dispute, deadline exceeded */
  --accent-red-dim:   rgba(239, 68, 68, 0.08);
  --accent-blue:      #3B82F6;   /* linki, info, tx hash */
  --accent-blue-dim:  rgba(59, 130, 246, 0.08);

  /* === Typografia === */
  --font-sans: 'Geist', 'DM Sans', sans-serif;
  --font-mono: 'Geist Mono', 'JetBrains Mono', monospace;

  /* === Border Radius === */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-modal: 12px;
}
```

---

## 3. Stack Technologiczny

```
Vite 5 + React 18 + TypeScript (strict)
Tailwind CSS v3
shadcn/ui  →  Card, Badge, Progress, Dialog, Tooltip, Separator, Skeleton
Framer Motion  →  animacje state machine
@solana/wallet-adapter-react  →  Connect Wallet
@solana/web3.js  →  getParsedAccountInfo (live Escrow balance)
lucide-react  →  ikony (wyłącznie wariant outline)
date-fns  →  countdown timery
```

**Tailwind config — dodaj do `tailwind.config.ts`:**
```ts
extend: {
  colors: {
    accent: 'var(--accent-primary)',
    surface: 'var(--bg-surface)',
    elevated: 'var(--bg-elevated)',
  },
  fontFamily: {
    sans: ['var(--font-sans)'],
    mono: ['var(--font-mono)'],
  },
}
```

---

## 4. Architektura Ekranów

### Priorytet budowy (hackathon order):

```
[P0] Task Detail Page     ← killer screen, demo starts here
[P1] Task Board (Kanban)  ← jeśli zostanie czas
[P2] Post Task Form       ← jeśli zostanie czas
[P3] Agent Browse View    ← jeśli zostanie czas
```

### Struktura routingu:

```
/            → redirect do /board
/board       → Kanban Task Board (dwa taby: Client / Agent)
/task/:id    → Task Detail Page [P0]
/task/new    → Post Task Form
```

---

## 5. [P0] Task Detail Page — Killer Screen

To jest najważniejszy ekran. Sędzia spędza tu 3 minuty. Musi pokazać wszystko.

### Layout (Desktop — 2 kolumny):

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back to Board          [CLAIMED] badge      [Wallet: 7xKp…] │
├──────────────────────────────┬──────────────────────────────────┤
│                              │                                  │
│  TASK HEADER                 │  ESCROW VAULT PANEL              │
│  Title (xl, --text-primary)  │  Total: 450 USDC                 │
│  Category badge              │  [==Advance==|======Main======]  │
│  Posted: 2h ago · by 3xAb…   │  Advance: 50 USDC (released ✓)  │
│                              │  Main:   400 USDC (locked 🔒)    │
│  DESCRIPTION                 │                                  │
│  [task description text]     │  COUNTDOWN TIMER                 │
│                              │  ⏱ 04 : 23 : 11                  │
│  REQUIREMENTS                │                                  │
│  • req 1                     │  TX HISTORY                      │
│  • req 2                     │  ● Escrow funded   3h ago        │
│  • req 3                     │  ● Agent claimed   1h ago        │
│                              │  ● Advance sent    1h ago        │
│  PROOF OF WORK               │                                  │
│  [Preview content area]      │  PRIMARY CTA                     │
│  ZK Proof: [hash] ✓verified  │  [Approve & Release 400 USDC]   │
│                              │  [Dispute]                       │
└──────────────────────────────┴──────────────────────────────────┘
```

### Layout (Mobile — single column):
Kolejność: Header → Countdown → Escrow Vault → Task Info → Proof → CTA sticky bottom

---

## 6. Komponenty — Specyfikacje Szczegółowe

### 6.1 `<TaskStatusBadge status={TaskStatus} />`

| Status | Tło | Tekst | Dot |
|---|---|---|---|
| `OPEN` | `--bg-elevated` | `--text-secondary` | szary, statyczny |
| `CLAIMED` | `--accent-amber-dim` | `--accent-amber` | amber, pulsujący |
| `IN_PROGRESS` | `--accent-amber-dim` | `--accent-amber` | amber, pulsujący |
| `PREVIEW_SUBMITTED` | `--accent-blue-dim` | `--accent-blue` | niebieski, pulsujący |
| `APPROVED` | `--accent-primary-dim` | `--accent-primary` | zielony, statyczny |
| `PAID` | `--accent-primary-dim` | `--accent-primary` | brak |
| `DISPUTED` | `--accent-red-dim` | `--accent-red` | czerwony, pulsujący |

Dot pulsujący (CSS):
```css
@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.3; }
}
.dot-pulse { animation: pulse-dot 1.5s ease-in-out infinite; }
```

### 6.2 `<EscrowVaultPanel />`

Główny panel wizualizacji środków. Kluczowy element designu.

```
┌─────────────────────────────────────┐
│  ESCROW VAULT              [icon]   │
│  ─────────────────────────────────  │
│                                     │
│  450.00 USDC                        │  ← font-mono, 32px, text-primary
│  vault: 8kNm...3pQr  [copy] [link]  │  ← font-mono, xs, text-muted
│                                     │
│  [██████░░░░░░░░░░░░░░░░░░░░░░░░░]  │  ← progress bar dwukolorowy
│   Advance (50)        Main (400)    │
│   Released ✓          Locked 🔒     │
│                                     │
│  ─────────────────────────────────  │
│  Last updated: 4s ago   [●] live    │  ← zielona kropka animowana
└─────────────────────────────────────┘
```

Progress bar: lewa sekcja `--accent-amber` (advance), prawa `--accent-primary` (main), separator 1px white.

Live polling: `getParsedAccountInfo` co 5s. Animuj zmianę salda spring animation.

### 6.3 `<CountdownTimer deadline={Date} />`

```
┌─────────────────────────────────────┐
│  SUBMIT DEADLINE                    │
│                                     │
│  04  :  23  :  11                   │  ← font-mono, 48px
│  hrs    min    sec                  │  ← font-sans, 11px, --text-muted
│                                     │
│  Oct 14, 2025 · 18:00 UTC           │  ← absolutna data, xs
└─────────────────────────────────────┘
```

Stany kolorów:
- `> 2h` → `--text-primary`
- `< 2h` → `--accent-amber`
- `< 15min` → `--accent-red` + pulsowanie komponentu
- `Expired` → `--accent-red`, tekst "DEADLINE EXCEEDED"

### 6.4 `<ProofOfWorkPanel />`

Widoczny gdy status = `PREVIEW_SUBMITTED` lub wyżej.

```
┌─────────────────────────────────────┐
│  PROOF OF WORK                      │
│  ─────────────────────────────────  │
│  ┌───────────────────────────────┐  │
│  │  [Preview — iframe / kod]     │  │  ← max-h: 300px, scroll
│  └───────────────────────────────┘  │
│                                     │
│  ZK Proof Hash                      │
│  [0x4a3f...c92b]  ✓ VERIFIED        │  ← font-mono + ShieldCheck badge
│                                     │
│  Result TX: [a8d3...11fe]  ↗        │  ← link Solana Explorer
└─────────────────────────────────────┘
```

ZK Verified badge: tło `--accent-primary-dim`, tekst `--accent-primary`, ikona `ShieldCheck`.

### 6.5 `<PrimaryActionPanel persona={Persona} status={TaskStatus} />`

Matryca akcji — renderuj właściwe CTA:

| Persona | Status | Primary | Secondary |
|---|---|---|---|
| Klient | `OPEN` | Fund Escrow | Edit Task |
| Klient | `CLAIMED` | — (waiting) | Cancel Task |
| Klient | `PREVIEW_SUBMITTED` | Approve & Release | Dispute |
| Klient | `APPROVED` | — | View Receipt |
| Agent | `OPEN` | Claim Task | — |
| Agent | `CLAIMED` | Submit Proof | Release Claim |
| Agent | `PREVIEW_SUBMITTED` | — (awaiting review) | — |
| Agent | `APPROVED` | — | View Payment |

Primary button style:
```css
background: var(--accent-primary);
color: var(--text-inverse);
border-radius: var(--radius-md);
font-weight: 600;
padding: 12px 24px;
transition: background 150ms ease;
/* hover: --accent-primary-hover */
/* disabled: opacity 0.4 */
```

Dispute button style:
```css
background: transparent;
color: var(--accent-red);
border: 1px solid var(--accent-red-dim);
/* hover: background --accent-red-dim */
```

---

## 7. [P1] Kanban Task Board

Header: dwa taby — **Client View** | **Agent View** — z animowanym aktywnym indykatorem.

Kolumny:
```
[  OPEN  ]  [  CLAIMED  ]  [  IN REVIEW  ]  [  COMPLETED  ]
```

### `<TaskCard />`:

```
┌─────────────────────────────────┐
│  [OPEN]  Smart Contract Audit   │  ← badge + title
│                                 │
│  Analyze Solana program for     │  ← 2 linie max, ellipsis
│  reentrancy vulnerabilities…    │
│                                 │
│  ─────────────────────────────  │
│  💰 450 USDC    ⏱ 6h left       │  ← font-mono
│  by: 8xKp…3mNr                  │  ← font-mono, xs
└─────────────────────────────────┘
```

Animacja przejścia między kolumnami: Framer Motion `layout="position"`, duration 250ms.

---

## 8. [P2] Post Task Form

Dwuetapowy wizard.

**Step 1 — Task Details:**
```
Title         [________________________________]
Category      [dropdown: Research/Audit/Data/Code/Other]
Description   [textarea, min 100 chars]
Requirements  [dynamic list, + Add Requirement]
```

**Step 2 — Budget & AI Pricing:**
```
┌──────────────────────────────────────────────┐
│  🤖 AI Price Estimate                         │
│  ──────────────────────────────────────────  │
│  Based on 47 similar tasks on-chain:         │
│  Suggested: 380 – 520 USDC                   │
│  Complexity: ████████░░  High                │
│  Est. time:  4 – 8 hours                     │
│                                              │
│  [────────────────●──────────] 450 USDC      │  ← slider
└──────────────────────────────────────────────┘

Advance (10–30%):  [──●──────────] 20% = 90 USDC

[← Back]                [Post Task & Fund Escrow →]
```

AI Estimate loading: skeleton pulsujący + "Analyzing similar tasks…" przez ~1.5s.

---

## 9. Typografia — Reguły

```
Page title:       font-sans, 24px, weight 600, --text-primary
Section heading:  font-sans, 12px, weight 600, --text-muted, UPPERCASE, tracking-widest
Body:             font-sans, 14px, weight 400, --text-secondary
Caption:          font-sans, 12px, weight 400, --text-muted
On-chain data:    font-mono, rozmiar kontekstowy, --text-primary
CTA button:       font-sans, 14px, weight 600
Amount (large):   font-mono, 32px, weight 700, --text-primary
Amount (inline):  font-mono, 14px, weight 500, --accent-primary
```

---

## 10. Animacje — Specyfikacja

```ts
// Page enter
initial: { opacity: 0, y: 8 }
animate: { opacity: 1, y: 0 }
transition: { duration: 0.2, ease: 'easeOut' }

// Status badge zmiana (AnimatePresence)
scale: 0.9 → 1.0 + opacity, duration 150ms

// Escrow balance live update
spring({ stiffness: 100, damping: 30 })  // animowany licznik

// Kanban card przejście
layout="position" na każdej karcie

// CTA button press
scale: 1 → 0.97 → 1, duration 100ms

// Approved state
Zielona fala / glow na EscrowVaultPanel, 500ms ease-out
```

---

## 11. Stany Ładowania i Błędów

**Skeleton loading:** shadcn `<Skeleton>` dla każdego panelu, kolor `--bg-elevated` → `--bg-overlay`.

**Empty states:**
```
Agent / brak zadań:
  Inbox icon (48px, --text-muted)
  "No open tasks"
  "New tasks will appear here."

Client / brak zadań:
  Plus icon
  "No tasks yet"
  CTA: [Post your first task]
```

**Error states:**
```
RPC Error:           "⚠ Live data unavailable · Using cached"
Wallet disconnected: Banner u góry z [Connect Wallet]
TX failed:           Toast bottom-right, czerwony, auto-dismiss 5s
```

**Toast notifications (bottom-right):**
```
Success:  lewa krawędź --accent-primary,  ikona CheckCircle
Error:    lewa krawędź --accent-red,      ikona XCircle
Info:     lewa krawędź --accent-blue,     ikona Info
Warning:  lewa krawędź --accent-amber,    ikona AlertTriangle
```

---

## 12. Responsywność

```
Mobile  (<768px):   single column, sticky bottom CTA bar
Tablet (768–1024):  2 kolumny (main + sidebar)
Desktop (>1024px):  layout 2 kolumny jak w specyfikacji Task Detail
```

**Sticky CTA mobile:**
```
Przyspięty do dołu, bg --bg-surface + backdrop-blur
Border-top: 1px --border-subtle
Padding: 16px
Full-width primary button
```

---

## 13. Wallet Connect

Zawsze widoczny w prawym górnym rogu headera.

```
Rozłączony:  [Connect Wallet]  ← outline button, small

Połączony:   [◉ 7xKp…3mNr ▾]  ← font-mono, dropdown
```

Dropdown:
```
┌──────────────────────┐
│  7xKpQm…3mNrAb       │
│  Balance: 12.4 SOL   │
│  ─────────────────── │
│  Copy Address        │
│  View on Explorer ↗  │
│  ─────────────────── │
│  Disconnect          │
└──────────────────────┘
```

---

## 14. Persona Switcher (Demo Mode)

Widoczny w headerze dla demonstracji hackathonowej:

```
[👤 Client]  [🤖 Agent]   ← toggle, animowany indykator
```

Zmiana persony przeładowuje widok z fade animacją. Widoczny gdy portfel nie jest podłączony LUB gdy tryb demo jest aktywny.

---

## 15. Folder Structure

```
src/
├── components/
│   ├── task/
│   │   ├── TaskStatusBadge.tsx
│   │   ├── TaskCard.tsx
│   │   ├── TaskDetailPage.tsx
│   │   └── ProofOfWorkPanel.tsx
│   ├── escrow/
│   │   ├── EscrowVaultPanel.tsx
│   │   └── CountdownTimer.tsx
│   ├── board/
│   │   ├── KanbanBoard.tsx
│   │   └── KanbanColumn.tsx
│   ├── actions/
│   │   └── PrimaryActionPanel.tsx
│   ├── layout/
│   │   ├── Header.tsx
│   │   ├── PersonaSwitcher.tsx
│   │   └── WalletButton.tsx
│   └── ui/              ← shadcn (nie edytuj)
├── hooks/
│   ├── useEscrowBalance.ts   ← polling getParsedAccountInfo
│   ├── useCountdown.ts
│   └── useTaskState.ts
├── types/
│   └── index.ts
├── styles/
│   └── globals.css           ← CSS variables z sekcji 2
├── lib/
│   ├── solana.ts
│   └── mockData.ts
└── App.tsx
```

---

## 16. Mock Data

```ts
// src/lib/mockData.ts

export const MOCK_TASK = {
  id: "task_7xKpQm",
  title: "Smart Contract Security Audit",
  category: "Audit",
  description: "Analyze the provided Solana program for reentrancy vulnerabilities, integer overflows, and unauthorized account access patterns.",
  requirements: [
    "Full reentrancy analysis",
    "Integer overflow detection",
    "Access control review",
    "Written report with severity ratings",
  ],
  status: "PREVIEW_SUBMITTED" as TaskStatus,
  totalAmount: 450,
  advanceAmount: 50,
  currency: "USDC",
  deadline: new Date(Date.now() + 4 * 60 * 60 * 1000 + 23 * 60 * 1000),
  clientAddress: "8xKpQm3mNrAbCd4eFgHiJk5lMnOpQrSt",
  agentAddress:  "3mNrAb7xKpQmCd4eFgHiJk5lMnOpQrSt",
  escrowAddress: "Esc1234abcd5678efgh9012ijkl3456mn",
  proofHash: "0x4a3fc2d1e8b9f07a2c5d6e1f3a8b2c9d4e7f0a1b",
  zkVerified: true,
  txHistory: [
    { event: "Escrow funded",    txHash: "a8d3...11fe", timestamp: new Date(Date.now() - 3 * 3600_000) },
    { event: "Agent claimed",    txHash: "b9e4...22gh", timestamp: new Date(Date.now() - 1 * 3600_000) },
    { event: "Advance released", txHash: "c0f5...33ij", timestamp: new Date(Date.now() - 58 * 60_000) },
    { event: "Proof submitted",  txHash: "d1g6...44kl", timestamp: new Date(Date.now() - 12 * 60_000) },
  ],
};
```

---

## 17. Definicje TypeScript

```ts
// src/types/index.ts

export type TaskStatus =
  | 'OPEN' | 'CLAIMED' | 'IN_PROGRESS'
  | 'PREVIEW_SUBMITTED' | 'APPROVED' | 'PAID' | 'DISPUTED';

export type Persona = 'client' | 'agent';

export type TaskCategory = 'Research' | 'Audit' | 'Data' | 'Code' | 'Other';

export interface Task {
  id: string;
  title: string;
  category: TaskCategory;
  description: string;
  requirements: string[];
  status: TaskStatus;
  totalAmount: number;
  advanceAmount: number;
  currency: 'USDC' | 'SOL';
  deadline: Date;
  clientAddress: string;
  agentAddress?: string;
  escrowAddress: string;
  proofHash?: string;
  zkVerified?: boolean;
  txHistory: TxEvent[];
}

export interface TxEvent {
  event: string;
  txHash: string;
  timestamp: Date;
}

export interface EscrowData {
  totalAmount: number;
  advanceReleased: number;
  mainLocked: number;
  balance: number;   // live z Solana RPC
  lastUpdated: Date;
}
```

---

## 18. Checklist Implementacji

```
[ ] CSS Variables w globals.css
[ ] Typy TypeScript (types/index.ts)
[ ] Mock data (lib/mockData.ts)
[ ] TaskStatusBadge — wszystkie 7 stanów
[ ] EscrowVaultPanel — statyczny
[ ] CountdownTimer — z kolorami stanów
[ ] PrimaryActionPanel — matryca akcji
[ ] TaskDetailPage — layout 2-kolumnowy
[ ] Live Escrow polling (useEscrowBalance)
[ ] ProofOfWorkPanel + ZK badge
[ ] Header + WalletButton + PersonaSwitcher
[ ] KanbanBoard + KanbanCard + animacje
[ ] Mobile responsywność + sticky CTA
[ ] PostTaskForm + AI estimate step
[ ] Toast notifications
```

---

> **Ostatnia zasada:** Jeśli decyzja projektowa nie jest opisana w tym dokumencie,
> wybierz opcję prostszą, ciemniejszą i bardziej monospacową.
> Gdy masz wątpliwości — zapytaj, nie zgaduj.