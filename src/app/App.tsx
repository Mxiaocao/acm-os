import { useEffect, useState } from "react";
import { getFoundationStatus, type FoundationStatusDto } from "../ipc/foundation";

export function App() {
  const [foundation, setFoundation] = useState<FoundationStatusDto | null>(null);
  const [error, setError] = useState<string | null>(null);

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
          setError(cause instanceof Error ? cause.message : String(cause));
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
      <p>This screen only proves the B0.1 repository and IPC boundaries.</p>
      <section aria-live="polite" className="status-card">
        {foundation ? (
          <p>
            Core status: <strong>{foundation.status}</strong> ({foundation.core})
          </p>
        ) : error ? (
          <p>Core unavailable: {error}</p>
        ) : (
          <p>Checking authoritative core...</p>
        )}
      </section>
    </main>
  );
}
