<script lang="ts">
  import { page } from "$app/stores";
  import Signup from "../../lib/ui/Login.svelte";
</script>

<Signup id={$page.params.id} />
