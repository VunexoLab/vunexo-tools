/**
 * Minimal inline-SVG icon set — hand-rolled, bundled at build time, no icon
 * font or CDN (same offline-first discipline as the sibling apps). Outline
 * style, 24x24 viewBox, `currentColor` stroke so every icon inherits its
 * container's text color/theme automatically.
 */
import type { SVGProps } from "react";

export type IconProps = SVGProps<SVGSVGElement>;

const base = {
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.5,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

export function SecretsIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <circle cx="8" cy="15" r="4.25" />
      <path d="M11 12.25 19.25 4M16.5 6.75l2.25 2.25M13.75 9.5 16 11.75" />
    </svg>
  );
}

export function EnvironmentsIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <path d="M12 3.75 20 8l-8 4.25L4 8Z" />
      <path d="M4 12l8 4.25L20 12" />
      <path d="M4 16l8 4.25L20 16" />
    </svg>
  );
}

export function ScanIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <circle cx="10.5" cy="10.5" r="6.25" />
      <path d="M15.25 15.25 20 20" />
    </svg>
  );
}

export function HooksIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <circle cx="6" cy="6" r="2.25" />
      <circle cx="6" cy="18" r="2.25" />
      <circle cx="18" cy="9" r="2.25" />
      <path d="M6 8.25V15.75" />
      <path d="M6 12c0-3 3-3.5 6-3.5s6.25-0.25 6-3" />
    </svg>
  );
}

export function SettingsIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82A1.65 1.65 0 0 0 3 13.09H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
    </svg>
  );
}

export function SunIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2.75v2M12 19.25v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2.75 12h2M19.25 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
    </svg>
  );
}

export function MoonIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <path d="M20.25 14.5A8.25 8.25 0 1 1 9.5 3.75a6.75 6.75 0 0 0 10.75 10.75Z" />
    </svg>
  );
}

export function LockIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <rect x="5" y="10.5" width="14" height="9.75" rx="1.5" />
      <path d="M8 10.5V7.5a4 4 0 0 1 8 0v3" />
    </svg>
  );
}

export function FolderIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <path d="M4 6.5a1 1 0 0 1 1-1h4.5l2 2.25H19a1 1 0 0 1 1 1V17.5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1Z" />
    </svg>
  );
}

export function EyeIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <path d="M2.75 12S6 5.75 12 5.75 21.25 12 21.25 12 18 18.25 12 18.25 2.75 12 2.75 12Z" />
      <circle cx="12" cy="12" r="2.5" />
    </svg>
  );
}

export function EyeOffIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <path d="M3.5 3.5l17 17" />
      <path d="M10.6 5.9A9.7 9.7 0 0 1 12 5.75c6 0 9.25 6.25 9.25 6.25a13.6 13.6 0 0 1-3.28 4.06M6.6 6.7A13.9 13.9 0 0 0 2.75 12S6 18.25 12 18.25a10 10 0 0 0 3.4-.6" />
      <path d="M9.9 10.1a2.5 2.5 0 0 0 3.5 3.5" />
    </svg>
  );
}

export function CopyIcon(props: IconProps) {
  return (
    <svg {...base} {...props}>
      <rect x="9" y="9" width="11" height="11" rx="1.5" />
      <path d="M6 15H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v1" />
    </svg>
  );
}
