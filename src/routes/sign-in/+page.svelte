<script lang="ts">
  import { page } from "$app/stores";
  import Login from "../../lib/ui/Login.svelte";
</script>

<Login id={$page.params.id} />
