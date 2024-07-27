<script lang="ts">
    import { options } from '../stores/options';
  
    function handleChange(key: string, value: boolean) {
      options.setOption(key as any, value);
    }
  </script>
  
  <div class="options-section">
    <h2 class="section-title">General Options</h2>
    <div class="space-y-4">
      <label class="flex items-center space-x-3 text-gray-700 dark:text-gray-300">
        <input
          type="checkbox"
          checked={$options.simplifiedMaterialView}
          on:change={() => handleChange('simplifiedMaterialView', !$options.simplifiedMaterialView)}
          class="form-checkbox h-5 w-5 text-blue-600"
        />
        <span>Simplified Study Material View</span>
      </label>
    </div>
  </div>