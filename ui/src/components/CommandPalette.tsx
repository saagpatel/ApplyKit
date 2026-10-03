import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { scoreCommand } from "../lib/command-score.mjs";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNavigate: (view: string) => void;
}

const entries = [
  { value: "dashboard", label: "Go to Dashboard" },
  { value: "new-job", label: "Go to New Job" },
  { value: "job-review", label: "Go to Job Review" },
  { value: "banks", label: "Go to Banks" },
  { value: "templates", label: "Go to Templates" },
  { value: "settings", label: "Go to Settings" }
];

export function CommandPalette({ open, ...props }: Props) {
  return open ? <OpenCommandPalette {...props} /> : null;
}

function OpenCommandPalette({ onOpenChange, onNavigate }: Omit<Props, "open">) {
  const [search, setSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const dialogRef = useRef<HTMLDialogElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const previousActiveRef = useRef<HTMLElement | null>(null);
  const id = useId();
  const listId = `${id}-commands`;

  // Keep the original label filter, followed by cmdk's value scoring and ordering.
  const filtered = useMemo(() => entries
    .filter((entry) => entry.label.toLowerCase().includes(search.toLowerCase()))
    .map((entry) => ({ ...entry, score: search ? scoreCommand(entry.value, search) : 1 }))
    .filter((entry) => entry.score > 0)
    .sort((a, b) => b.score - a.score), [search]);
  const active = filtered[activeIndex];

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    // Native modal inertness does not prevent wheel/trackpad background scrolling.
    const root = document.documentElement;
    const body = document.body;
    const scrollStyles = [[root, "overflow-x"], [root, "overflow-y"], [body, "overflow-x"], [body, "overflow-y"], [body, "padding-right"]] as const;
    const previousScrollStyles = scrollStyles.map(([element, property]) => ({
      element, property, value: element.style.getPropertyValue(property), priority: element.style.getPropertyPriority(property)
    }));
    const scrollbarGap = window.innerWidth - root.clientWidth;
    if (scrollbarGap > 0) body.style.paddingRight = `${parseFloat(getComputedStyle(body).paddingRight) + scrollbarGap}px`;
    root.style.overflow = "hidden";
    body.style.overflow = "hidden";
    if (!dialog.open) {
      previousActiveRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      dialog.showModal();
      inputRef.current?.focus();
    }
    return () => {
      for (const { element, property, value, priority } of previousScrollStyles) element.style.setProperty(property, value, priority);
      if (dialog.open) {
        dialog.close();
        previousActiveRef.current?.focus({ preventScroll: true });
      }
    };
  }, []);

  useEffect(() => {
    if (active) document.getElementById(`${id}-${active.value}`)?.scrollIntoView({ block: "nearest" });
  }, [active, id]);

  const close = () => {
    onOpenChange(false);
  };
  const select = (value: string) => {
    onNavigate(value);
    close();
  };
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.nativeEvent.isComposing || event.keyCode === 229) return;
    const down = event.key === "ArrowDown" || (event.ctrlKey && ["j", "n"].includes(event.key));
    const up = event.key === "ArrowUp" || (event.ctrlKey && ["k", "p"].includes(event.key));
    if (down || up || event.key === "Home" || event.key === "End") {
      event.preventDefault();
      const last = Math.max(0, filtered.length - 1);
      setActiveIndex((index) => event.key === "Home" || (up && event.metaKey) ? 0
        : event.key === "End" || (down && event.metaKey) ? last
        : Math.max(0, Math.min(last, index + (down ? 1 : -1))));
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (active) select(active.value);
    }
  };

  return (
    <dialog
      ref={dialogRef}
      className="palette"
      aria-labelledby={`${id}-title`}
      aria-describedby={`${id}-description`}
      onCancel={(event) => { event.preventDefault(); close(); }}
      onClose={() => {
        // A queued close event from StrictMode must not close a freshly reopened dialog.
        if (!dialogRef.current?.open) {
          onOpenChange(false);
          previousActiveRef.current?.focus({ preventScroll: true });
        }
      }}
      onKeyDown={(event) => {
        // The input is the only tab stop; retain the palette's original focus loop.
        if (event.key === "Tab") {
          event.preventDefault();
          inputRef.current?.focus();
        }
      }}
      onClick={(event) => {
        if (event.target !== event.currentTarget) return;
        const rect = event.currentTarget.getBoundingClientRect();
        if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) close();
      }}
    >
      <h2 id={`${id}-title`} className="sr-only">Command palette</h2>
      <p id={`${id}-description`} className="sr-only">Type to find a command, then press Enter to navigate.</p>
      <input
        ref={inputRef}
        role="combobox"
        aria-label="Search commands"
        aria-expanded="true"
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={active ? `${id}-${active.value}` : undefined}
        placeholder="Type a command..."
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
        value={search}
        onChange={(event) => { setSearch(event.target.value); setActiveIndex(0); }}
        onKeyDown={onKeyDown}
        className="palette-input"
      />
      <div id={listId} role="listbox" aria-label="Commands">
        {filtered.map((entry, index) => (
          <div
            key={entry.value}
            id={`${id}-${entry.value}`}
            role="option"
            aria-selected={index === activeIndex}
            data-selected={index === activeIndex}
            onPointerMove={() => setActiveIndex(index)}
            onClick={() => select(entry.value)}
            className="palette-item"
          >{entry.label}</div>
        ))}
      </div>
      {filtered.length === 0 && <p role="status" className="palette-item">No results.</p>}
    </dialog>
  );
}
