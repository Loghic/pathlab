# Architecture

This document explains how the crate is organised and how data flows
through it at runtime.

## Module map

```mermaid
graph TD
    main["main.rs<br/>(native + wasm entry)"] --> app
    subgraph app["app/"]
        state["state.rs<br/>MazeApp (eframe::App)"]
        top["top_bar.rs"]
        side["side_panel.rs"]
        canvas["canvas.rs"]
        state --> top
        state --> side
        state --> canvas
    end

    app --> solver
    app --> mazes
    app --> platform

    subgraph solver["solver/"]
        algo["algorithm.rs<br/>Algorithm enum"]
        heur["heuristic.rs<br/>Heuristic enum"]
        sv["core.rs<br/>Solver"]
        kp["k_paths.rs<br/>Yen's algorithm"]
        sv --> algo
        sv --> heur
        sv --> kp
    end

    subgraph mazes["mazes/"]
        cell["cell.rs<br/>Cell, MazeGrid"]
        gen["generators.rs<br/>built-in mazes"]
        pbm["pbm.rs<br/>P1 reader/writer"]
        gen --> cell
        pbm --> cell
    end

    subgraph platform["platform/"]
        time["time.rs<br/>Instant via web-time"]
        fileio["fileio.rs<br/>desktop / web pickers"]
    end

    solver --> mazes
    platform --> mazes
```

Each module owns exactly one concern:

- **`mazes/`** — what a maze is and how to make / load / save one.
- **`solver/`** — how to search for a path. Knows nothing about UI.
- **`platform/`** — abstracts the two things that genuinely differ
  between native and wasm: monotonic time and file pickers.
- **`app/`** — egui front-end. Renders state, dispatches user actions.

## Update loop

`MazeApp::update` runs once per egui frame and does four things, in order:

```mermaid
sequenceDiagram
    participant Egui
    participant App as MazeApp::update
    participant TopBar as top_bar::show
    participant Side as side_panel::show
    participant Canvas as canvas::show
    participant Tick as tick_solver
    participant Solver

    Egui->>App: redraw frame
    App->>TopBar: render File menu
    App->>Side: render settings panel
    App->>Canvas: render maze + overlay
    App->>Tick: drain file inbox, advance solver
    Tick->>Solver: step() if speed_ms elapsed
    Tick-->>App: want_repaint?
    App-->>Egui: request_repaint() if needed
```

`tick_solver` is intentionally pulled out of the side-panel callback so
the timing logic is testable and the UI code stays declarative.

## How a solve happens

```mermaid
sequenceDiagram
    participant User
    participant Side as Side panel
    participant App as MazeApp
    participant Solver
    participant Canvas

    User->>Side: click "Solve"
    Side->>App: start_solve()
    App->>Solver: Solver::new(algo, heur, start, goal)
    loop while !finished and speed elapsed
        App->>Solver: step(&maze)
        Solver-->>App: open_cells / closed_cells / path updated
        App->>Canvas: redraw with overlay
    end
    Solver-->>App: status = Found | NoPath
```

## The web timer fix

The earlier version of this project called `std::time::Instant::now()`
directly. That panics on `wasm32-unknown-unknown` because the platform
has no built-in monotonic clock.

`platform::time` re-exports
[`web-time::Instant`](https://docs.rs/web-time), which:

- on native = `std::time::Instant`,
- on wasm = backed by `performance.now()`.

Every timing-sensitive call site (`last_step_time`, `last_finish_time`)
goes through this module, so the same code drives both runtimes.

```mermaid
flowchart LR
    A[App needs Instant::now] --> B{cfg target_arch}
    B -- not wasm32 --> C[std::time::Instant]
    B -- wasm32 --> D[web-time uses performance.now]
    C --> E[Same API]
    D --> E
```

## State ownership

```mermaid
classDiagram
    class MazeApp {
        +MazeGrid maze
        +Solver? solver
        +Algorithm algorithm
        +Heuristic heuristic
        +Point start, end
        +InteractionMode interaction
        +FileInbox file_inbox
        +tick_solver()
        +start_solve()
    }
    class Solver {
        -Algorithm algorithm
        -Heuristic heuristic
        -BinaryHeap open_heap
        -VecDeque open_queue
        -Vec open_stack
        -HashSet closed
        +step(maze)
        +status()
        +path()
    }
    class Heuristic {
        Manhattan
        Euclidean
        Chebyshev
        Zero
        +estimate(a, b) f32
    }
    class FileInbox {
        +put(result)
        +take() Option
    }

    MazeApp --> Solver : owns
    Solver --> Heuristic : uses
    MazeApp --> FileInbox : owns
```

Only `MazeApp` is mutable at the top level — `Solver` borrows the maze
immutably during each `step()`, so wall edits and solver iteration cannot
race.
