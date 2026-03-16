"use client";

import { useMemo, useRef, useState } from "react";
import { FaGithub, FaChevronRight } from "react-icons/fa";
import { SiRust } from "react-icons/si";
import { VscVscode } from "react-icons/vsc";
import { FiAlertCircle, FiAlertTriangle } from "react-icons/fi";

export default function Home() {
  const fileRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [rawResult, setRawResult] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [severityFilter, setSeverityFilter] = useState<string | null>(null);
  const backendBaseUrl =
    process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://127.0.0.1:7070";

  const examplePayload = {
    report: {
      ruleset_version: "0.8.2",
      generated_at_unix: 1710604800,
      total_duration_ms: 1240,
      cache_stats: {
        nested_bundles: { hits: 2, misses: 1 },
        usage_scan: { hits: 0, misses: 4 },
        private_api_scan: { hits: 0, misses: 1 },
      },
      results: [
        {
          rule_id: "RULE_XCODE_26_MANDATE",
          rule_name: "Xcode 26 / iOS 26 SDK Mandate",
          category: "Compliance",
          severity: "Error",
          status: "Fail",
          message: "App was built with Xcode 15.4 (15F31d) and iOS 17.5 SDK",
          recommendation: "From April 2026, all apps must be built with Xcode 26 and the iOS 26 SDK.",
          duration_ms: 5,
        },
        {
          rule_id: "RULE_PRIVACY_MANIFEST",
          rule_name: "Missing Privacy Manifest",
          category: "Privacy",
          severity: "Error",
          status: "Fail",
          message: "PrivacyInfo.xcprivacy was not found in the main bundle",
          recommendation: "Add a PrivacyInfo.xcprivacy file to your app bundle to declare data collection and and accessed APIs.",
          duration_ms: 8,
        },
        {
          rule_id: "RULE_PRIVACY_SDK_CROSSCHECK",
          rule_name: "Privacy Manifest vs SDK Usage",
          category: "Privacy",
          severity: "Error",
          status: "Fail",
          message: "Detected GoogleAnalytics and FirebaseSDK but they are not declared in the manifest.",
          recommendation: "Ensure PrivacyInfo.xcprivacy declares data collection and accessed APIs for all included third-party SDKs.",
          duration_ms: 450,
        },
        {
          rule_id: "RULE_ENTITLEMENTS_MISMATCH",
          rule_name: "Debug Entitlements Present",
          category: "Entitlements",
          severity: "Error",
          status: "Fail",
          message: "Found get-task-allow=true in app entitlements",
          recommendation: "Remove the get-task-allow entitlement for App Store production builds.",
          duration_ms: 15,
        },
        {
          rule_id: "RULE_BUNDLE_RESOURCE_LEAKAGE",
          rule_name: "Sensitive Files in Bundle",
          category: "Bundle",
          severity: "Error",
          status: "Fail",
          message: "Found .env and development.p12 inside the app bundle",
          recommendation: "Remove certificates, provisioning profiles, or secret files from the app bundle before submission.",
          duration_ms: 25,
        },
        {
          rule_id: "RULE_CAMERA_USAGE",
          rule_name: "Missing Camera Usage Description",
          category: "Privacy",
          severity: "Error",
          status: "Fail",
          message: "NSCameraUsageDescription is missing from Info.plist",
          recommendation: "Add NSCameraUsageDescription with a clear, user-facing reason why your app needs camera access.",
          duration_ms: 4,
        },
        {
          rule_id: "RULE_ATS_AUDIT",
          rule_name: "ATS Exceptions Detected",
          category: "Ats",
          severity: "Warning",
          status: "Fail",
          message: "NSAllowsArbitraryLoads is enabled globally",
          recommendation: "Remove global ATS exceptions or scope them to specific domains with strong justification.",
          duration_ms: 12,
        },
        {
          rule_id: "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT",
          rule_name: "LSApplicationQueriesSchemes Audit",
          category: "Metadata",
          severity: "Warning",
          status: "Fail",
          message: "Found 5+ potentially generic or private schemes in allowlist",
          recommendation: "Keep LSApplicationQueriesSchemes minimal and aligned with actual app handoff requirements.",
          duration_ms: 10,
        },
        {
          rule_id: "RULE_EXPORT_COMPLIANCE",
          rule_name: "Export Compliance Declaration",
          category: "Metadata",
          severity: "Warning",
          status: "Fail",
          message: "ITSAppUsesNonExemptEncryption is not set",
          recommendation: "Explicitly set ITSAppUsesNonExemptEncryption in Info.plist to avoid App Store Connect prompts.",
          duration_ms: 5,
        },
        {
          rule_id: "RULE_PRIVATE_API",
          rule_name: "Private API Usage Detected",
          category: "Private API",
          severity: "Warning",
          status: "Fail",
          message: "Potential usage of _GSSystemAdditions detected in binary",
          recommendation: "Remove private API usage or replace with public alternatives to avoid rejection.",
          duration_ms: 600,
        }
      ],
    },
  };

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
    setStatus("Scanning...");
    setResult(null);

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch(`${backendBaseUrl}/api/v1/scan`, {
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

      const response = await fetch(`${backendBaseUrl}/api/v1/handoff`, {
        method: "POST",
        body: form,
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

    const byCategory = failures.reduce<Record<string, number>>((acc, item) => {
      const category = String(item.category ?? "Other");
      acc[category] = (acc[category] ?? 0) + 1;
      return acc;
    }, {});

    const bySeverity = failures.reduce<Record<string, number>>((acc, item) => {
      const severity = String(item.severity ?? "Unknown");
      acc[severity] = (acc[severity] ?? 0) + 1;
      return acc;
    }, {});

    const findingsByCategory = failures.reduce<Record<string, any[]>>((acc, item) => {
      const category = String(item.category ?? "Other");
      if (!acc[category]) acc[category] = [];
      acc[category].push(item);
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
      findingsByCategory,
    };
  }, [result]);

  const toggleCategory = (category: string) => {
    const next = new Set(expandedCategories);
    if (next.has(category)) {
      next.delete(category);
    } else {
      next.add(category);
    }
    setExpandedCategories(next);
  };

  const expandAll = (categories: string[]) => {
    setExpandedCategories(new Set(categories));
  };

  const collapseAll = () => {
    setExpandedCategories(new Set());
  };

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
                  <div className="result-header">
                    <span>Findings by severity</span>
                    {severityFilter && (
                      <button className="ghost-button" style={{ fontSize: "10px", padding: "2px 8px" }} onClick={() => setSeverityFilter(null)}>
                        Clear filter
                      </button>
                    )}
                  </div>
                  <div className="pill-row">
                    {Object.entries(summary.bySeverity).map(([name, count]) => (
                      <button
                        key={name}
                        className={`pill-chip pill-chip--${name.toLowerCase()} ${severityFilter === name ? "is-active" : ""}`}
                        onClick={() => setSeverityFilter(severityFilter === name ? null : name)}
                        style={{ border: severityFilter === name ? "1px solid currentColor" : "1px solid transparent", cursor: "pointer" }}
                      >
                        {name === "Error" ? <FiAlertCircle /> : <FiAlertTriangle />}
                        <span>{name}</span>
                        <strong>{count}</strong>
                      </button>
                    ))}
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">
                    <span>Findings Explorer</span>
                    <div className="pill-row">
                      <button className="ghost-button" style={{ fontSize: "10px", padding: "2px 8px" }} onClick={() => expandAll(Object.keys(summary.findingsByCategory))}>
                        Expand all
                      </button>
                      <button className="ghost-button" style={{ fontSize: "10px", padding: "2px 8px" }} onClick={collapseAll}>
                        Collapse all
                      </button>
                    </div>
                  </div>
                  <div className="tree-view">
                    {Object.entries(summary.findingsByCategory).sort().map(([category, rawItems]) => {
                      const items = severityFilter ? (rawItems as any[]).filter(i => i.severity === severityFilter) : (rawItems as any[]);
                      if (items.length === 0) return null;

                      const isExpanded = expandedCategories.has(category);
                      const catErrors = items.filter((i: any) => i.severity === "Error").length;
                      return (
                        <div key={category} className={`tree-node ${isExpanded ? "is-expanded" : ""}`}>
                          <div className="tree-header" onClick={() => toggleCategory(category)}>
                            <div className="tree-header-left">
                              <FaChevronRight className="tree-arrow" />
                              <span>{category}</span>
                            </div>
                            <div className="pill-row">
                              {catErrors > 0 && (
                                <span className="tree-badge" style={{ background: "rgba(217, 72, 72, 0.1)", color: "#b92c2c" }}>
                                  {catErrors} Errors
                                </span>
                              )}
                              <span className="tree-badge">
                                {items.length} {severityFilter ? severityFilter.toLowerCase() : "item"}s
                              </span>
                            </div>
                          </div>
                          {isExpanded && (
                            <div className="tree-content">
                              {items.map((item, idx) => (
                                <div key={idx} className="tree-finding">
                                  <div className="tree-finding-title">
                                    <strong>{item.rule_name}</strong>
                                    <span className={`pill-chip pill-chip--${String(item.severity).toLowerCase()}`} style={{ padding: "2px 8px", fontSize: "10px" }}>
                                      {item.severity}
                                    </span>
                                  </div>
                                  <div className="tree-finding-meta">
                                    <span>Rule: {item.rule_id}</span>
                                    {typeof item.duration_ms === "number" && (
                                      <span>{item.duration_ms}ms</span>
                                    )}
                                  </div>
                                  <div className="tree-finding-desc">
                                    {item.message}
                                  </div>
                                  {item.evidence && (
                                    <pre className="tree-finding-evidence">
                                      {typeof item.evidence === "string" 
                                        ? item.evidence 
                                        : JSON.stringify(item.evidence, null, 2)}
                                    </pre>
                                  )}
                                  {item.recommendation && (
                                    <div className="tree-finding-rec">
                                      {item.recommendation}
                                    </div>
                                  )}
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      );
                    })}
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
