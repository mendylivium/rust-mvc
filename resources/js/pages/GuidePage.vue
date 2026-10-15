<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">START HERE</p>
      <h1>Make it your own.</h1>
      <p class="muted">Your map from starter to application.</p>
    </div>
  </div>
  <div class="example-grid">
    <section class="card example-panel">
      <h2>1. Run the workspace</h2>
      <p>
        Copy .env.example to .env, choose a seed password, then run these commands from the project
        root.
      </p>
      <pre><code>npm ci
./forge migrate --seed
node scripts/dev.mjs</code></pre>
      <p>Open http://127.0.0.1:5173. Vite proxies API requests to Rust on port 3000.</p>
    </section>
    <section class="card example-panel">
      <h2>2. Explore the examples</h2>
      <p>Copy a pattern into your page and adapt its state and data.</p>
      <ul class="example-links">
        <li><RouterLink to="/examples/elements">Elements, buttons & dialogs</RouterLink></li>
        <li><RouterLink to="/examples/cards">Cards & metrics</RouterLink></li>
        <li><RouterLink to="/examples/tables">Searchable, sortable tables</RouterLink></li>
        <li><RouterLink to="/examples/forms">Forms & validation</RouterLink></li>
      </ul>
      <p>Examples are local demos. Users and Settings are backed by the Rust API.</p>
    </section>
    <section class="card example-panel">
      <h2>3. Follow the full tutorial</h2>
      <p>
        Open <code>docs/TUTORIAL.md</code> in the repository for a complete walkthrough: setup,
        request flow, a database-backed Projects feature, tests, and deployment.
      </p>
      <p>
        Backend: <code>app/http/controllers</code><br />Frontend: <code>resources/js/pages</code
        ><br />Theme: <code>resources/css/app.css</code>
      </p>
    </section>
    <section class="card example-panel">
      <h2>4. Verify your changes</h2>
      <pre><code>cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run build
npm run test:e2e</code></pre>
      <p>
        Register each page in Vue Router and routes/web.rs so direct production links return HTTP
        200.
      </p>
    </section>
  </div>
</template>
