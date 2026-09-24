<script lang="ts">
  let {
    id,
    name,
    label,
    rows = 3,
    placeholder = "",
    value = $bindable(""),
    onkeydown
  }: {
    id: string;
    name: string;
    label: string;
    rows?: number;
    placeholder?: string;
    value?: string;
    onkeydown?: (event: KeyboardEvent) => void;
  } = $props();

  let field: HTMLTextAreaElement | undefined;

  export function focus() {
    field?.focus();
  }
</script>

<label for={id} class="br-visually-hidden">{label}</label>
<textarea
  {id}
  {name}
  {rows}
  {placeholder}
  class="br-field"
  bind:value
  bind:this={field}
  {onkeydown}
></textarea>
