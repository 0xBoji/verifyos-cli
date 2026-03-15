export default function Home() {
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
              <button className="primary-button" type="button">
                Upload bundle
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
              <div className="drop-icon">⬆</div>
              <div>
                <strong>Drag &amp; drop your bundle</strong>
                <span>.ipa or .app, up to 1GB</span>
              </div>
              <button className="secondary-button" type="button">
                Choose file
              </button>
            </div>
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

        <section className="footer-card">
          <div>
            <h3>Ready for CI + agents</h3>
            <p>
              Connect verifyOS to your pipeline or workflow bot. We’ll keep your
              reviewers and AI assistants focused on the highest-impact fixes.
            </p>
          </div>
          <button className="primary-button" type="button">
            Connect backend
          </button>
        </section>
      </main>
    </div>
  );
}
