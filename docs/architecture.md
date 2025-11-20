# Architecture & Messaging

## Layer Communication

The game follows a strict layered architecture with unidirectional messaging:

```text
UI Layer (Godot) → Game Logic → Core Simulation
                 ← Commands   ← Events
```

## Sequence Diagrams

### Trial System Flow

```mermaid
sequenceDiagram
    participant UI as TrialGui
    participant Game as GameLogic
    participant Core as CoreSimulation
    participant Trial as TrialSystem

    Note over Core: Diner triggers feedback
    Core->>Game: SimEvent::TrialStart
    Game->>UI: UiCommand::TrialStart
    UI->>UI: Show trial UI

    Core->>Game: SimEvent::TrialIntro
    Game->>UI: UiCommand::TrialIntro
    UI->>UI: Display intro

    UI->>Game: GameRequest::TrialIntroDone
    Game->>Core: SimCommand::TrialIntroDone

    Core->>Game: SimEvent::TrialLeftSpeak
    Game->>UI: UiCommand::TrialLeftSpeak
    UI->>UI: Display diner speech

    UI->>Game: GameRequest::TrialCheckKeyword
    Game->>Core: SimCommand::TrialCheckKeyword

    Core->>Game: SimEvent::TrialResponseCandidates
    Game->>UI: UiCommand::TrialResponseCandidates
    UI->>UI: Show response options

    UI->>Game: GameRequest::TrialRespond
    Game->>Core: SimCommand::TrialRespond

    Core->>Game: SimEvent::TrialRightSpeak
    Game->>UI: UiCommand::TrialRightSpeak
    UI->>UI: Display manager response

    Core->>Game: SimEvent::TrialImpact
    Game->>UI: UiCommand::TrialImpact
    UI->>UI: Show reputation/mood changes

    Core->>Game: SimEvent::TrialEnd
    Game->>UI: UiCommand::TrialEnd
    UI->>UI: Close trial UI
```

### Settlement and Management Decision Flow

```mermaid
sequenceDiagram
    participant UI as SettlementGui/DecisionGui
    participant Game as GameLogic
    participant Core as CoreSimulation

    Note over Core: Run ends
    Core->>Game: SimEvent::RunCompleted(settlement)
    Game->>UI: UiCommand::ShowSettlement
    UI->>UI: Display settlement stats

    UI->>Game: GameRequest::ConfirmSettlement
    Game->>Core: SimCommand::ConfirmSettlement
    Core->>Core: Apply reputation changes
    Core->>Core: Check for endings

    alt Ending Triggered (reputation ≤0 or ≥100)
        Core->>Game: SimEvent::ShowEnding
        Game->>UI: UiCommand::ShowEnding
        UI->>UI: Display ending screen

        alt Good Ending (can continue)
            Core->>Game: SimEvent::ShowManagementDecisions
            Game->>UI: UiCommand::ShowDecisionSelection
            Note over UI: Decisions ready but hidden under ending

            UI->>Game: GameRequest::ContinueFromEnding
            Game->>UI: Hide ending, show decisions
            UI->>UI: Display decision options
        else Bad Ending (game over)
            UI->>Game: GameRequest::ExitLevel
            Note over UI: Return to main menu
        end
    else No Ending (moderate reputation)
        Core->>Game: SimEvent::ShowManagementDecisions
        Game->>UI: UiCommand::ShowDecisionSelection
        UI->>UI: Display decision options
    end

    UI->>Game: GameRequest::SelectDecision(index)
    Game->>Core: SimCommand::ApplyManagementDecision(index)

    Core->>Core: Apply permanent effects
    Core->>Core: Advance day
    Core->>Game: SimEvent::Persist
    Game->>Game: Save profile

    Core->>Game: SimEvent::DayCompleted
    Game->>UI: UiCommand::FinishDay
    UI->>UI: Transition to next day
```

### Dish Pricing Flow

```mermaid
sequenceDiagram
    participant UI as DishPricePopup
    participant Game as GameLogic
    participant Core as CoreSimulation

    UI->>UI: Click on dish
    UI->>Game: GameRequest::OpenDishEditor
    Game->>UI: UiCommand::OpenDishPriceEditor
    UI->>UI: Show pricing popup

    UI->>Game: GameRequest::ApplyDishPrice
    Game->>Core: SimCommand::SetDishPrice
    Core->>Core: Update dish pricing
```

### Ending Flow

```mermaid
sequenceDiagram
    participant UI as EndingGui
    participant Scene as GameScene
    participant Proc as ExitLevelProcedure
    participant Profile as ProfileService
    participant Core as CoreSimulation

    Note over Core: Reputation <= 0 or >= 100
    Core->>Core: on_confirm_settlement checks reputation
    Core->>Core: Trigger AchieveEnding event
    Core->>Scene: SimEvent::ShowEnding
    Scene->>Profile: Save ending to profile
    Scene->>UI: UiCommand::ShowEnding
    UI->>UI: Display ending screen

    alt Good Ending (can_continue=true)
        Note over Core: Also trigger RollManagementDecisions
        Core->>Scene: SimEvent::ShowManagementDecisions
        Scene->>UI: UiCommand::ShowDecisionSelection
        Note over UI: Decisions loaded but hidden

        UI->>Scene: GameRequest::ContinueFromEnding
        Scene->>UI: Hide ending, show decisions
        UI->>UI: Display decision options
        Note over UI: Player continues playing
    else Bad Ending (can_continue=false)
        UI->>Scene: AppRequest::ExitLevel
        Scene->>Scene: Set exiting_after_ending flag
        Scene->>Proc: Schedule ExitLevelProcedure
        Proc->>Profile: Clear level progress
        Proc->>Proc: Pop game scene
        Note over UI: Return to start menu
    end
```

## State Diagrams

### Diner State Transitions

```mermaid
stateDiagram-v2
    [*] --> Approaching: Spawn

    Approaching --> QueuingForTray: Reach dispenser
    Approaching --> Leaving: Too crowded

    QueuingForTray --> Ordering: Get tray
    QueuingForTray --> Leaving: Wait timeout

    Ordering --> MovingToTable: Complete order
    Ordering --> Leaving: Service timeout

    MovingToTable --> Seated: Find table
    MovingToTable --> Leaving: No table available

    Seated --> Eating: Start meal

    Eating --> EatingDone: Finish dishes
    Eating --> Leaving: Trial timeout/fail

    EatingDone --> InTrial: Triggered feedback
    EatingDone --> ReturningTray: No feedback

    InTrial --> ReturningTray: Trial complete
    InTrial --> Leaving: Trial timeout

    ReturningTray --> Leaving: Return dishes

    Leaving --> [*]: Despawn
```

### Day Phase Transitions

```mermaid
stateDiagram-v2
    [*] --> Preparation: Start day

    Preparation --> Running: Start run
    Preparation --> [*]: Exit level

    Running --> Settlement: Run completed
    Running --> [*]: Force end / Exit level

    Settlement --> EndingCheck: Confirm settlement
    Settlement --> [*]: Exit level

    EndingCheck --> Ending: Bad/Good ending triggered
    EndingCheck --> DecisionMaking: No ending (moderate reputation)
    EndingCheck --> [*]: Exit level

    Ending --> [*]: Exit (bad ending)
    Ending --> DecisionMaking: Continue (good ending)

    DecisionMaking --> Preparation: Select decision (next day)
    DecisionMaking --> [*]: Exit level
```

## Key Principles

1. **Unidirectional Flow**: UI never directly modifies game state
2. **Event-Driven**: Core emits events, UI responds with commands
3. **State Isolation**: Each layer owns its state
4. **Persistence Boundary**: Only Game layer interacts with ProfileService
5. **Progress Separation**: Level progress cleared on ending exit, profile data preserved

## Key Principles

1. **Unidirectional Flow**: UI never directly modifies game state
2. **Event-Driven**: Core emits events, UI responds with commands
3. **State Isolation**: Each layer owns its state
4. **Persistence Boundary**: Only Game layer interacts with ProfileService
5. **Progress Separation**: Level progress cleared on ending exit, profile data preserved
