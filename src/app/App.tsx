import { useEffect, useState } from "react";
import { getFoundationStatus, type FoundationStatusDto } from "../ipc/foundation";
import { getStartupStatus, type StartupStatusDto } from "../ipc/startup";

export function App() {
  const [foundation, setFoundation] = useState<FoundationStatusDto | null>(null);
  const [startup, setStartup] = useState<StartupStatusDto | null>(null);
  const [foundationError, setFoundationError] = useState<string | null>(null);
  const [startupError, setStartupError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    getFoundationStatus()
      .then((result) => {
        if (active) {
          setFoundation(result);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setFoundationError(cause instanceof Error ? cause.message : String(cause));
        }
      });

    getStartupStatus()
      .then((result) => {
        if (active) {
          setStartup(result);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setStartupError(cause instanceof Error ? cause.message : String(cause));
        }
      });

    return () => {
      active = false;
    };
  }, []);

  return (
    <main className="app-shell">
      <p className="eyebrow">ACM-OS</p>
      <h1>BUILD foundation</h1>
      <p>This screen proves the repository, IPC, and SQLite startup boundaries.</p>
      <section aria-live="polite" className="status-card">
        {foundation ? (
          <p>
            Core status: <strong>{foundation.status}</strong> ({foundation.core})
          </p>
        ) : foundationError ? (
          <p>Core unavailable: {foundationError}</p>
        ) : (
          <p>Checking authoritative core...</p>
        )}
        {startup?.state === "ready" ? (
          <p>
            Database startup gate: <strong>ready</strong> (schema {startup.schemaVersion})
          </p>
        ) : startup?.state === "recoveryRequired" ? (
          <p>
            Database startup gate: <strong>recovery required</strong> ({startup.recoveryReason})
          </p>
        ) : startupError ? (
          <p>Database startup gate unavailable: {startupError}</p>
        ) : (
          <p>Checking database startup gate...</p>
        )}
      </section>
    </main>
  );
}
