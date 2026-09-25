<script lang="ts">
  /**
   * B20-U4: the composer's editor. It grows from 60 px to 180 px — the pinned
   * `min-h-[60px] max-h-[180px]` range — and carries the combobox attributes
   * that announce the suggestion popover without moving focus out of the field.
   */
  let {
    id,
    name,
    label,
    rows = 1,
    placeholder = "",
    value = $bindable(""),
    minHeight = 60,
    maxHeight = 180,
    role = "textbox",
    expanded = undefined,
    controls = undefined,
    activeDescendant = undefined,
    autocomplete = undefined,
    onkeydown,
    oninput
  }: {
    id: string;
    name: string;
    label: string;
    rows?: number;
    placeholder?: string;
    value?: string;
    minHeight?: number;
    maxHeight?: number;
    role?: "textbox" | "combobox";
    expanded?: boolean;
    controls?: string;
    activeDescendant?: string;
    autocomplete?: "both" | "inline" | "list" | "none";
    onkeydown?: (event: KeyboardEvent) => void;
    oninput?: (event: Event) => void;
  } = $props();

  let field: HTMLTextAreaElement | undefined;

  export function focus() {
    field?.focus();
  }

  /** Used when the composer appends a trigger, so the caret lands after it. */
  export function focusEnd() {
    if (!field) {
      return;
    }
    field.focus();
    const end = field.value.length;
    field.setSelectionRange(end, end);
  }

  export function grow() {
    if (!field) {
      return;
    }
    field.style.height = "auto";
    field.style.height = `${Math.min(Math.max(field.scrollHeight, minHeight), maxHeight)}px`;
  }

  $effect(() => {
    // The value is the dependency: a programmatic change (a suggestion click or
    // the app clearing the draft after send) has to resize the field too.
    void value;
    grow();
  });
</script>

<label for={id} class="br-visually-hidden">{label}</label>
<textarea
  {id}
  {name}
  {rows}
  {placeholder}
  class="br-field composer-editor"
  {role}
  aria-multiline="true"
  aria-expanded={expanded}
  aria-controls={controls}
  aria-activedescendant={activeDescendant}
  aria-autocomplete={autocomplete}
  bind:value
  bind:this={field}
  onkeydown={onkeydown ? (event) => onkeydown(event) : undefined}
  oninput={(event) => {
    grow();
    oninput?.(event);
  }}
></textarea>

<style>
  .composer-editor {
    /* A textarea is inline-block by default, and its line box adds a few pixels
       of descender space under the field. The pinned editor is a block. */
    display: block;
    min-height: 60px;
    max-height: 180px;
    padding: 16px 16px 8px;
    border: 0;
    border-radius: 0;
    background: transparent;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    overflow-y: auto;
    resize: none;
  }
</style>
