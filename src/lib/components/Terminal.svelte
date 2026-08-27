<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  // import "@xterm/xterm/css/xterm.css";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  /** Arbeitsverzeichnis (Projektpfad). */
  export let cwd: string;
  /** Eindeutige Session-ID. */
  export let sessionId: string;

  let el: HTMLDivElement;
  let term: Terminal;
  let fit: FitAddon;
  let unlistenOutput: UnlistenFn | undefined;
  let unlistenClosed: UnlistenFn | undefined;
  let onWinResize: (() => void) | undefined;

  onMount(async () => {
    // xterm initialisieren.
    term = new Terminal({
      fontSize: 13,
      fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", monospace',
      cursorBlink: true,
      theme: {
        background: "#0a0a0a",
        foreground: "#e5e5e5",
        cursor: "#e5e5e5",
      },
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(el);
    fit.fit();

    // Output-Events vom Backend → ins Terminal schreiben.
    unlistenOutput = await listen<{ sessionId: string; data: string }>(
      "terminal-output",
      (event) => {
        if (event.payload.sessionId === sessionId) {
          term.write(event.payload.data);
        }
      },
    );

    // Session-Ende signalisieren.
    unlistenClosed = await listen<string>("terminal-closed", (event) => {
      if (event.payload === sessionId) {
        term.write("\r\n\x1b[90m[Prozess beendet]\x1b[0m\r\n");
      }
    });

    // Tastatureingaben → ans Backend.
    term.onData((data) => {
      invoke("cmd_terminal_write", { sessionId, data });
    });

    // Terminal-Resize → PTY-Größe anpassen.
    term.onResize(({ rows, cols }) => {
      invoke("cmd_terminal_resize", { sessionId, rows, cols });
    });

    // Backend-Session starten.
    await invoke("cmd_terminal_open", { sessionId, cwd });

    // Initiale Größe ans Backend melden.
    fit.fit();
    invoke("cmd_terminal_resize", {
      sessionId,
      rows: term.rows,
      cols: term.cols,
    });

    // Bei Fensteränderung neu anpassen.
    onWinResize = () => fit.fit();
    window.addEventListener("resize", onWinResize);

    term.focus();
  });

  onDestroy(() => {
    if (onWinResize) window.removeEventListener("resize", onWinResize);
    unlistenOutput?.();
    unlistenClosed?.();
    invoke("cmd_terminal_close", { sessionId }).catch(() => {});
    term?.dispose();
  });
</script>

<div bind:this={el} class="h-full w-full"></div>
