"use client";

import { useMemo, useRef, useState } from "react";

export default function Home() {
  const fileRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [rawResult, setRawResult] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);

  const handleChooseFile = () => {
    fileRef.current?.click();
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] ?? null;
    setSelectedFile(file);
    setStatus(file ? `Selected ${file.name}` : "No file selected");
    setResult(null);
    setRawResult(null);
  };

  const handleUpload = async () => {
    if (!selectedFile || isUploading) {
      return;
    }

    setIsUploading(true);
    setStatus("Uploading bundle...");
    setResult(null);

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch("http://127.0.0.1:7070/api/v1/scan", {
        method: "POST",
        body: form,
      });

      const rawText = await response.text();
      let payload: unknown = rawText;
      if (rawText) {
        try {
          payload = JSON.parse(rawText);
        } catch {
          payload = rawText;
        }
      }

      if (!response.ok) {
        const message =
          typeof payload === "object" && payload !== null && "error" in payload
            ? String((payload as { error?: string }).error)
            : `Scan failed (${response.status})`;
        setStatus(message);
        setResult(null);
        setRawResult(rawText || null);
        return;
      }

      setStatus("Scan complete");
      if (payload && typeof payload === "object") {
        setResult(payload as Record<string, unknown>);
        setRawResult(JSON.stringify(payload, null, 2));
      } else {
        setResult(null);
        setRawResult(rawText || null);
      }
    } catch (error) {
      setStatus("Failed to reach backend. Is it running on :7070?");
      setResult(null);
      setRawResult(null);
    } finally {
      setIsUploading(false);
    }
  };

  const summary = useMemo(() => {
    const report = result?.report as
      | { results?: Array<Record<string, unknown>>; total_duration_ms?: number }
      | undefined;
    const results = report?.results ?? [];
    const failures = results.filter((item) => {
      const status = item.status as string | undefined;
      return status === "Fail" || status === "Error";
    });
    const errorCount = failures.filter((item) => item.severity === "Error").length;
    const warningCount = failures.filter((item) => item.severity === "Warning").length;
    const duration =
      typeof report?.total_duration_ms === "number"
        ? `${report.total_duration_ms}ms`
        : null;

    return {
      results,
      failures,
      errorCount,
      warningCount,
      duration,
    };
  }, [result]);

  return (
    <div className="page">
      <div className="page-glow page-glow--left" />
      <div className="page-glow page-glow--right" />

      <header className="nav">
        <div className="logo">
          <span className="logo-mark" aria-hidden="true">
            ✓
          </span>
          <div>
            <div className="logo-title">verifyOS</div>
            <div className="logo-subtitle">App Store review confidence</div>
          </div>
        </div>
        <div className="nav-actions">
          <button className="ghost-button" type="button">
            Docs
          </button>
          <button className="primary-button" type="button">
            New Scan
          </button>
        </div>
      </header>

      <main className="shell">
        <section className="hero">
          <div className="hero-copy">
            <div className="badge">iOS-friendly diagnostics</div>
            <h1>
              Ship App Store reviews with{" "}
              <span className="accent">zero surprises</span>.
            </h1>
            <p>
              Upload your <span className="pill">.ipa</span> or{" "}
              <span className="pill">.app</span> and get a clean, structured
              report for privacy, entitlements, signing, and metadata risks.
              Designed for AI agents and human reviewers.
            </p>
            <div className="hero-actions">
              <button className="primary-button" type="button" onClick={handleChooseFile}>
                Choose bundle
              </button>
              <button className="secondary-button" type="button">
                View example report
              </button>
            </div>
            <div className="hero-meta">
              <div>
                <strong className="stat">2-4 min</strong>
                <span>Typical scan time</span>
              </div>
              <div>
                <strong className="stat">0%</strong>
                <span>Data leaves device</span>
              </div>
              <div>
                <strong className="stat">JSON/SARIF</strong>
                <span>Agent-ready output</span>
              </div>
            </div>
          </div>

          <div className="hero-card">
            <div className="card-header">
              <div>
                <h3>Quick Scan</h3>
                <span>Best for pre-submit checks</span>
              </div>
              <span className="chip">Profile: Full</span>
            </div>
            <div className="dropzone">
              <div className="dropzone-content">
                <div className="drop-icon">⬆</div>
                <strong>Drag &amp; drop your bundle</strong>
                <span>.ipa or .app, up to 1GB</span>
              </div>
              <input
                ref={fileRef}
                className="file-input"
                type="file"
                accept=".ipa,.app"
                onChange={handleFileChange}
                hidden
              />
              <button className="secondary-button" type="button" onClick={handleChooseFile}>
                Choose file
              </button>
            </div>
            <div className="upload-actions">
              <button
                className="primary-button"
                type="button"
                onClick={handleUpload}
                disabled={!selectedFile || isUploading}
              >
                {isUploading ? "Uploading..." : "Run scan"}
              </button>
              <div className="upload-status">
                {selectedFile ? selectedFile.name : "No file selected"}
              </div>
            </div>
            {status ? <div className="status-pill">{status}</div> : null}
            {result ? (
              <div className="report-stack">
                <div className="report-summary">
                  <div>
                    <div className="summary-label">Errors</div>
                    <div className="summary-value">{summary.errorCount}</div>
                  </div>
                  <div>
                    <div className="summary-label">Warnings</div>
                    <div className="summary-value">{summary.warningCount}</div>
                  </div>
                  <div>
                    <div className="summary-label">Findings</div>
                    <div className="summary-value">{summary.failures.length}</div>
                  </div>
                  <div>
                    <div className="summary-label">Duration</div>
                    <div className="summary-value">{summary.duration ?? "—"}</div>
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">Top findings</div>
                  <ul className="finding-list">
                    {summary.failures.slice(0, 5).map((item, index) => (
                      <li key={`${item.rule_id ?? "rule"}-${index}`}>
                        <strong>{String(item.rule_name ?? "Untitled rule")}</strong>
                        <span>{String(item.recommendation ?? "Review this rule")}</span>
                      </li>
                    ))}
                    {summary.failures.length === 0 ? (
                      <li className="finding-empty">No failing rules detected.</li>
                    ) : null}
                  </ul>
                </div>

                <div className="result-card">
                  <div className="result-header">Report actions</div>
                  <div className="report-actions">
                    <button
                      className="secondary-button"
                      type="button"
                      onClick={() => {
                        if (!rawResult) return;
                        const blob = new Blob([rawResult], { type: "application/json" });
                        const url = URL.createObjectURL(blob);
                        const link = document.createElement("a");
                        link.href = url;
                        link.download = "verifyos-report.json";
                        link.click();
                        URL.revokeObjectURL(url);
                      }}
                    >
                      Download JSON
                    </button>
                    <button
                      className="ghost-button"
                      type="button"
                      onClick={() => {
                        if (!rawResult) return;
                        void navigator.clipboard?.writeText(rawResult);
                        setStatus("Report copied to clipboard");
                      }}
                    >
                      Copy JSON
                    </button>
                  </div>
                </div>

                {rawResult ? (
                  <details className="result-card">
                    <summary className="result-header">Raw report</summary>
                    <pre>{rawResult}</pre>
                  </details>
                ) : null}
              </div>
            ) : null}
            <div className="card-footer">
              <div>
                <strong>Next:</strong> privacy manifest, entitlements, ATS rules
              </div>
              <button className="ghost-button" type="button">
                Advanced options
              </button>
            </div>
          </div>
        </section>

        <section className="steps">
          <div className="step">
            <div className="step-number">1</div>
            <div>
              <h4>Upload bundle</h4>
              <p>Scan locally with zero external uploads or cloud storage.</p>
            </div>
          </div>
          <div className="step">
            <div className="step-number">2</div>
            <div>
              <h4>Review findings</h4>
              <p>
                Clear severity, evidence, and recommendations for each rule.
              </p>
            </div>
          </div>
          <div className="step">
            <div className="step-number">3</div>
            <div>
              <h4>Hand off to AI</h4>
              <p>Generate agent packs, PR comments, and fix prompts in one tap.</p>
            </div>
          </div>
        </section>

      </main>
    </div>
  );
}
