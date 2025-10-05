<script lang=ts>
  import characterImage from "../../../assets/chara/01.png"
</script>

<div class="tab-content">
  <div class="setting-section">
    <div class="section-header">
      <h3>Support StudyCraft</h3>
    </div>
    <p>StudyCraft is forever free and open-source. If you like it, you can support its development by donating <a href="https://ko-fi.com/rodmarkun">here</a>.</p>
    
    <div class="support-image">
      <img src="{characterImage}" alt="StudyCraft character" />
    </div>
  </div>
</div>

<style>
  .tab-content {
    height: 100%;
  }

  .setting-section {
    margin-bottom: var(--space-xl);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-lg);
    padding-bottom: var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .section-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .support-image {
    display: flex;
    justify-content: center;
    margin-top: var(--space-sm);
  }

  .support-image img {
    max-width: 300px;
    height: auto;
    border-radius: var(--border-radius);
    opacity: 0.9;
  }
</style>