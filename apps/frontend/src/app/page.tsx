"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { FaGithub } from "react-icons/fa";
import { SiRust } from "react-icons/si";
import { VscVscode } from "react-icons/vsc";

export default function Home() {
  const fileRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [rawResult, setRawResult] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [authEmail, setAuthEmail] = useState("");
  const [authCode, setAuthCode] = useState("");
  const [authToken, setAuthToken] = useState<string | null>(null);
  const [authStatus, setAuthStatus] = useState<string | null>(null);
  const [authBusy, setAuthBusy] = useState(false);

  const examplePayload = {
    report: {
      ruleset_version: "0.8.1",
      generated_at_unix: 0,
      total_duration_ms: 842,
      cache_stats: {
        nested_bundles: { hits: 1, misses: 0 },
        usage_scan: { hits: 2, misses: 0 },
        private_api_scan: { hits: 0, misses: 1 },
      },
      results: [
        {
          rule_id: "RULE_PRIVACY_MANIFEST",
          rule_name: "Missing Privacy Manifest",
          category: "Privacy",
          severity: "Error",
          status: "Fail",
          message: "Missing PrivacyInfo.xcprivacy",
          recommendation: "Add a PrivacyInfo.xcprivacy manifest to the bundle.",
          duration_ms: 12,
        },
        {
          rule_id: "RULE_USAGE_DESCRIPTIONS",
          rule_name: "Missing Usage Description Keys",
          category: "Privacy",
          severity: "Warning",
          status: "Fail",
          message: "Missing required usage description keys",
          recommendation: "Add NS*UsageDescription keys to Info.plist.",
          duration_ms: 9,
        },
        {
          rule_id: "RULE_ATS_GRANULARITY",
          rule_name: "ATS Exceptions Too Broad",
          category: "Ats",
          severity: "Warning",
          status: "Fail",
          message: "AllowsArbitraryLoads is enabled",
          recommendation: "Scope ATS exceptions to specific domains.",
          duration_ms: 8,
        },
      ],
    },
  };

  const handleChooseFile = () => {
    fileRef.current?.click();
  };

  useEffect(() => {
    const token = localStorage.getItem("verifyos_auth_token");
    const email = localStorage.getItem("verifyos_auth_email");
    if (token) {
      setAuthToken(token);
    }
    if (email) {
      setAuthEmail(email);
    }
  }, []);

  const authHeaders = authToken
    ? {
        Authorization: `Bearer ${authToken}`,
      }
    : undefined;

  const handleAuthStart = async () => {
    if (!authEmail || authBusy) {
      return;
    }
    setAuthBusy(true);
    setAuthStatus("Sending login code...");
    try {
      const response = await fetch("http://127.0.0.1:7070/api/v1/auth/start", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email: authEmail }),
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        setAuthStatus(
          typeof payload?.error === "string" ? payload.error : "Login failed"
        );
        return;
      }
      if (payload?.dev_code) {
        setAuthStatus(`Code: ${payload.dev_code}`);
      } else {
        setAuthStatus("Check your inbox for the login code.");
      }
    } catch (error) {
      setAuthStatus("Failed to reach auth service.");
    } finally {
      setAuthBusy(false);
    }
  };

  const handleAuthVerify = async () => {
    if (!authEmail || !authCode || authBusy) {
      return;
    }
    setAuthBusy(true);
    setAuthStatus("Verifying code...");
    try {
      const response = await fetch("http://127.0.0.1:7070/api/v1/auth/verify", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email: authEmail, code: authCode }),
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        setAuthStatus(
          typeof payload?.error === "string" ? payload.error : "Invalid code"
        );
        return;
      }
      if (payload?.token) {
        setAuthToken(payload.token);
        localStorage.setItem("verifyos_auth_token", payload.token);
        localStorage.setItem("verifyos_auth_email", authEmail);
        setAuthStatus("Signed in.");
        setAuthCode("");
      } else {
        setAuthStatus("Invalid response from auth service.");
      }
    } catch (error) {
      setAuthStatus("Failed to verify code.");
    } finally {
      setAuthBusy(false);
    }
  };

  const handleAuthSignOut = () => {
    setAuthToken(null);
    setAuthStatus("Signed out.");
    localStorage.removeItem("verifyos_auth_token");
    localStorage.removeItem("verifyos_auth_email");
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
    setStatus("Scanning...");
    setResult(null);

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch("http://127.0.0.1:7070/api/v1/scan", {
        method: "POST",
        body: form,
        headers: authHeaders,
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

  const handleDownloadBundle = async () => {
    if (!selectedFile || isDownloading) {
      return;
    }

    setIsDownloading(true);
    setStatus("Preparing agent bundle...");

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch("http://127.0.0.1:7070/api/v1/handoff", {
        method: "POST",
        body: form,
        headers: authHeaders,
      });

      if (!response.ok) {
        const text = await response.text();
        setStatus(text || `Bundle failed (${response.status})`);
        return;
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "verifyos-handoff.zip";
      link.click();
      URL.revokeObjectURL(url);
      setStatus("Agent bundle downloaded");
    } catch (error) {
      setStatus("Failed to download agent bundle");
    } finally {
      setIsDownloading(false);
    }
  };

  const handleExampleReport = () => {
    setStatus("Loaded example report");
    setSelectedFile(null);
    setResult(examplePayload as Record<string, unknown>);
    setRawResult(JSON.stringify(examplePayload, null, 2));
  };

  const summary = useMemo(() => {
    const report = result?.report as
      | {
          results?: Array<Record<string, unknown>>;
          total_duration_ms?: number;
        }
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

    const byCategory = results.reduce<Record<string, number>>((acc, item) => {
      const category = String(item.category ?? "Other");
      acc[category] = (acc[category] ?? 0) + 1;
      return acc;
    }, {});

    const bySeverity = results.reduce<Record<string, number>>((acc, item) => {
      const severity = String(item.severity ?? "Unknown");
      acc[severity] = (acc[severity] ?? 0) + 1;
      return acc;
    }, {});

    return {
      results,
      failures,
      errorCount,
      warningCount,
      duration,
      byCategory,
      bySeverity,
    };
  }, [result]);

  return (
    <div className="page">
      <div className="page-glow page-glow--left" />
      <div className="page-glow page-glow--right" />

      <header className="nav">
        <div className="logo">
          <span className="logo-mark" aria-hidden="true">
            <img src="/logo/verifyOS_web_128x.png" alt="" />
          </span>
          <div>
            <div className="logo-title">verifyOS</div>
            <div className="logo-subtitle">App Store review confidence</div>
          </div>
        </div>
        <div className="nav-actions">
          <a
            className="ghost-button"
            href="https://github.com/0xBoji/verifyOS#readme"
            target="_blank"
            rel="noreferrer"
          >
            Docs
          </a>
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
              Scan <span className="pill">.ipa</span>,{" "}
              <span className="pill">.app</span>,{" "}
              <span className="pill">.xcodeproj</span>, or{" "}
              <span className="pill">.xcworkspace</span> (zip) and get a clean,
              structured report for privacy, entitlements, signing, metadata,
              and more. Designed for AI agents and human reviewers.
            </p>
          </div>
        </section>

        <section className="auth-panel">
          <div className="auth-card">
            <div>
              <h3>Email login</h3>
              <p>Enable rate-limited scans and keep audit access tied to you.</p>
            </div>
            {authToken ? (
              <div className="auth-actions">
                <div className="auth-status">Signed in as {authEmail}</div>
                <button className="ghost-button" type="button" onClick={handleAuthSignOut}>
                  Sign out
                </button>
              </div>
            ) : (
              <div className="auth-actions">
                <input
                  className="auth-input"
                  type="email"
                  placeholder="you@company.com"
                  value={authEmail}
                  onChange={(event) => setAuthEmail(event.target.value)}
                />
                <div className="auth-row">
                  <button
                    className="secondary-button"
                    type="button"
                    onClick={handleAuthStart}
                    disabled={authBusy || !authEmail}
                  >
                    Send code
                  </button>
                  <input
                    className="auth-input auth-input--code"
                    type="text"
                    placeholder="Code"
                    value={authCode}
                    onChange={(event) => setAuthCode(event.target.value)}
                  />
                  <button
                    className="primary-button"
                    type="button"
                    onClick={handleAuthVerify}
                    disabled={authBusy || !authCode}
                  >
                    Verify
                  </button>
                </div>
                {authStatus ? <div className="auth-status">{authStatus}</div> : null}
              </div>
            )}
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

        <section className="scan-panel" id="quick-scan">
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
                    <div className="summary-value summary-value--error">
                      {summary.errorCount}
                    </div>
                  </div>
                  <div>
                    <div className="summary-label">Warnings</div>
                    <div className="summary-value summary-value--warning">
                      {summary.warningCount}
                    </div>
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
                  <div className="result-header">Findings by category</div>
                  <div className="bar-list">
                    {Object.entries(summary.byCategory).map(([name, count]) => (
                      <div key={name} className="bar-row">
                        <span>{name}</span>
                        <div className="bar">
                          <div
                            className="bar-fill"
                            style={{ width: `${Math.min(count * 12, 100)}%` }}
                          />
                        </div>
                        <strong>{count}</strong>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">Findings by severity</div>
                  <div className="pill-row">
                    {Object.entries(summary.bySeverity).map(([name, count]) => (
                      <div
                        key={name}
                        className={`pill-chip pill-chip--${name.toLowerCase()}`}
                      >
                        <span>{name}</span>
                        <strong>{count}</strong>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">Report actions</div>
                  <div className="report-actions">
                    <button
                      className="secondary-button"
                      type="button"
                      onClick={handleDownloadBundle}
                      disabled={!selectedFile || isDownloading}
                    >
                      {isDownloading ? "Preparing bundle..." : "Download agent bundle"}
                    </button>
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
                      className={`ghost-button copy-button ${copied ? "is-copied" : ""}`}
                      type="button"
                      onClick={() => {
                        if (!rawResult) return;
                        void navigator.clipboard?.writeText(rawResult);
                        setCopied(true);
                        setStatus("Copied JSON to clipboard");
                        window.setTimeout(() => setCopied(false), 1500);
                      }}
                    >
                      <span className="copy-icon" aria-hidden="true" />
                      {copied ? "Copied" : "Copy JSON"}
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
              <button className="ghost-button" type="button" onClick={handleExampleReport}>
                View example report
              </button>
            </div>
          </div>
        </section>

        <footer className="app-footer">
          <div>
            <div className="footer-label">
              verifyOS
            </div>
            <div className="footer-subtitle">Resources &amp; downloads</div>
          </div>
          <nav className="footer-links" aria-label="verifyOS links">
            <a
              href="https://github.com/0xBoji/verifyOS"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <FaGithub className="footer-icon" aria-hidden="true" />
              GitHub Repo
            </a>
            <a
              href="https://marketplace.visualstudio.com/items?itemName=0xBoji.verifyos-vscode"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <VscVscode className="footer-icon" aria-hidden="true" />
              VS Code Extension
            </a>
            <a
              href="https://crates.io/crates/verifyos-cli"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <SiRust className="footer-icon" aria-hidden="true" />
              crates.io
            </a>
          </nav>
        </footer>
      </main>
    </div>
  );
}
