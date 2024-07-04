<script>
  let count = 0
  const increment = () => {
    count += 1
  }
</script>

<button on:click={increment} class="p-4 mt-12 border border-indigo-900 border-opacity-80 rounded-md hover:border-indigo-500 hover:bg-slate-800 transition-all duration-300">
  count is {count}
</button>
