import { useState } from "react";
import { EnvironmentsIcon, HooksIcon, MoonIcon, ScanIcon, SecretsIcon, SettingsIcon, SunIcon } from "../components/icons";
import { EnvironmentsScreen } from "../features/environments/EnvironmentsScreen";
import { HooksScreen } from "../features/hooks/HooksScreen";
import { OpenProjectGate } from "../features/project/OpenProjectGate";
import { UnlockGate } from "../features/project/UnlockGate";
import { ScanScreen } from "../features/scan/ScanScreen";
import { SecretsScreen } from "../features/secrets/SecretsScreen";
import { SettingsScreen } from "../features/settings/SettingsScreen";
import { useTheme } from "../hooks/useTheme";

// gui-ux.md §"Navigation structure" — flat, six sections, same shape as the
// sibling apps' sidebar.
type Section = "secrets" | "environments" | "scan" | "hooks" | "settings";

const SECTIONS: { id: Section; label: string; icon: (props: { className?: string }) => JSX.Element }[] = [
  { id: "secrets", label: "Secrets", icon: SecretsIcon },
  { id: "environments", label: "Environments", icon: EnvironmentsIcon },
  { id: "scan", label: "Scan", icon: ScanIcon },
  { id: "hooks", label: "Hooks", icon: HooksIcon },
  { id: "settings", label: "Settings", icon: SettingsIcon },
];

function folderNameOf(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? path;
}

interface OpenProject {
  path: string;
  unlocked: boolean;
}

/**
 * gui-ux.md's navigation structure: a fixed left sidebar (project-initial
 * badge + open project's folder name, nav buttons with a left accent bar on
 * the active item, theme toggle pinned at the bottom) plus a centered
 * content pane. The sidebar only renders once a project is open and
 * unlocked — until then, the whole window is the Open Project / Unlock gate
 * (§1/§2), matching the sibling apps' full-screen-gate pattern.
 */
export function App() {
  const [project, setProject] = useState<OpenProject | null>(null);
  const [section, setSection] = useState<Section>("secrets");
  const { theme, toggle } = useTheme();

  if (!project) {
    return (
      <OpenProjectGate
        onProjectOpened={(path, { unlocked }) => {
          setProject({ path, unlocked });
          setSection("secrets");
        }}
      />
    );
  }

  if (!project.unlocked) {
    return (
      <UnlockGate
        projectPath={project.path}
        onUnlocked={() => setProject((p) => (p ? { ...p, unlocked: true } : p))}
      />
    );
  }

  const folderName = folderNameOf(project.path);
  const initial = folderName.charAt(0).toUpperCase() || "V";

  return (
    <div className="flex min-h-screen bg-background text-text-primary">
      <nav className="flex w-56 shrink-0 flex-col border-r border-border bg-surface p-4">
        <div className="mb-6 flex items-center gap-2 px-2">
          <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-accent text-sm font-semibold text-white">
            {initial}
          </span>
          <span className="truncate text-sm font-semibold" title={project.path}>
            {folderName}
          </span>
        </div>
        <ul className="flex-1 space-y-0.5">
          {SECTIONS.map((s) => {
            const Icon = s.icon;
            const active = section === s.id;
            return (
              <li key={s.id}>
                <button
                  onClick={() => setSection(s.id)}
                  className={`relative flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm transition-colors ${
                    active
                      ? "bg-accent/10 font-medium text-accent"
                      : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
                  }`}
                >
                  {active && <span className="absolute -left-4 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-full bg-accent" />}
                  <Icon className="h-4 w-4 shrink-0" />
                  {s.label}
                </button>
              </li>
            );
          })}
        </ul>
        <button
          onClick={toggle}
          className="mt-4 flex items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm text-text-secondary transition-colors hover:bg-surface-hover hover:text-text-primary"
        >
          {theme === "dark" ? <SunIcon className="h-4 w-4" /> : <MoonIcon className="h-4 w-4" />}
          {theme === "dark" ? "Light mode" : "Dark mode"}
        </button>
      </nav>
      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-5xl px-8 py-8">
          {section === "secrets" && <SecretsScreen />}
          {section === "environments" && <EnvironmentsScreen />}
          {section === "scan" && <ScanScreen />}
          {section === "hooks" && <HooksScreen />}
          {section === "settings" && (
            <SettingsScreen
              onLocked={() => {
                setProject((p) => (p ? { ...p, unlocked: false } : p));
                setSection("secrets");
              }}
            />
          )}
        </div>
      </main>
    </div>
  );
}
