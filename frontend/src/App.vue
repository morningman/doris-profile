<template>
  <div id="app" class="app-container">
    <header class="app-header">
      <div class="header-content">
        <div class="logo">
          <i class="fas fa-chart-line"></i>
          <h1>Doris Profile Analyzer</h1>
        </div>
        <nav class="nav-links">
          <a href="https://doris.apache.org" target="_blank" rel="noopener">
            <i class="fas fa-external-link-alt"></i> Doris Docs
          </a>
          <a
            href="https://github.com/apache/doris"
            target="_blank"
            rel="noopener"
          >
            <i class="fab fa-github"></i> GitHub
          </a>
        </nav>
      </div>
    </header>

    <main class="app-main">
      <router-view />
    </main>

    <footer class="app-footer">
      <p>
        Doris Profile Analyzer v0.1.0 | Made for Apache Doris Community
      </p>
    </footer>
  </div>
</template>

<script>
export default {
  name: "App",
};
</script>

<style lang="scss">
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-gradient);
}

.app-header {
  background: var(--header-bg);
  border-bottom: 1px solid var(--border-color);
  padding: 0 24px;
  height: 64px;
  display: flex;
  align-items: center;
  backdrop-filter: blur(10px);
  position: sticky;
  top: 0;
  z-index: 100;

  .header-content {
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;

    i {
      font-size: 28px;
      color: var(--primary-color);
    }

    h1 {
      font-size: 20px;
      font-weight: 600;
      color: var(--text-primary);
      margin: 0;
    }
  }

  .nav-links {
    display: flex;
    gap: 24px;

    a {
      color: var(--text-secondary);
      text-decoration: none;
      font-size: 14px;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: color 0.2s;

      &:hover {
        color: var(--primary-color);
      }
    }
  }
}

.app-main {
  flex: 1;
  padding: 24px;
  max-width: 1400px;
  width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
}

.app-footer {
  background: var(--header-bg);
  border-top: 1px solid var(--border-color);
  padding: 16px 24px;
  text-align: center;

  p {
    margin: 0;
    font-size: 13px;
    color: var(--text-tertiary);
  }
}
</style>

