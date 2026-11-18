import { Monitor, Moon, Settings, Sun } from "lucide-react";
import { version } from "../../package.json";
import BannedExtEditor from "../features/rules/BannedExtEditor";
import Segmented from "./Segmented";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { Theme } from "../hooks/useTheme";

interface SettingsMenuProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  /** Yasakli uzanti listesi (F-B). */
  bannedExtensions: string[];
  onAddBanned: (raw: string) => void;
  onRemoveBanned: (ext: string) => void;
  onResetBanned: () => void;
}

const THEME_TABS: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Açık", icon: Sun },
  { value: "dark", label: "Koyu", icon: Moon },
  { value: "auto", label: "Otomatik", icon: Monitor },
];

/** Ayar menusu (tema + yasakli uzantilar). Radix Popover: Esc / disari tiklama kapatir, odak tetikleyiciye doner. */
export default function SettingsMenu({
  theme,
  onThemeChange,
  bannedExtensions,
  onAddBanned,
  onRemoveBanned,
  onResetBanned,
}: SettingsMenuProps) {
  return (
    <Popover>
      <PopoverTrigger className="btn" title="Ayarlar" aria-label="Ayarlar">
        <Settings size={16} />
      </PopoverTrigger>
      <PopoverContent align="end" className="menu-pop" aria-label="Ayarlar">
        <p className="menu-label" id="theme-label">Görünüm</p>
        <Segmented value={theme} options={THEME_TABS} onChange={onThemeChange} labelledBy="theme-label" />

        <div className="menu-section">
          <BannedExtEditor
            extensions={bannedExtensions}
            onAdd={onAddBanned}
            onRemove={onRemoveBanned}
            onReset={onResetBanned}
          />
        </div>

        <div className="menu-section">
          <p className="menu-note">Smart File Organizer · v{version}</p>
          <p className="menu-note">Tema ve yasaklı uzantılar bu cihazda saklanır.</p>
        </div>
      </PopoverContent>
    </Popover>
  );
}
