import { browser } from "$app/environment";

export type Language = "en" | "ja";
export type Theme = "light" | "dark" | "system";

const translations = {
  en: {
    live: "Live",
    recordings: "Recordings",
    settings: "Settings",
    language: "Language",
    theme: "Theme",
    english: "English",
    japanese: "Japanese",
    light: "Light",
    dark: "Dark",
    system: "System",
    recordingNotEnabled: "Recording is not enabled",
  },
  ja: {
    live: "ライブ",
    recordings: "録画",
    settings: "設定",
    language: "言語",
    theme: "テーマ",
    english: "英語",
    japanese: "日本語",
    light: "ライト",
    dark: "ダーク",
    system: "システム",
    recordingNotEnabled: "録画は無効化されています",
  },
};

class Settings {
  language = $state<Language>("ja");
  theme = $state<Theme>("system");

  constructor() {
    if (browser) {
      const savedLang = localStorage.getItem("language") as Language;
      if (savedLang) {
        this.language = savedLang;
      } else {
        const browserLang = navigator.language;
        this.language = browserLang.startsWith("ja") ? "ja" : "en";
      }

      const savedTheme = localStorage.getItem("theme") as Theme;
      if (savedTheme) this.theme = savedTheme;
    }
  }

  get t() {
    return translations[this.language];
  }
}

export const settings = new Settings();
