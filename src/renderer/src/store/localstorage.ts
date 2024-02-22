import { IStorageService } from './storage';

export const LOCALSTORAGE_MENU_OPEN_KEYS = 'menu_open_keys';
export const LOCALSTORAGE_CURRENT_FILE_ID = 'current_file_id';
export const LOCALSTORAGE_BOARD_CUSTOM_FONTS = 'custom_fonts';
export const LOCALSTORAGE_LANG_CODE = 'lang_code';
export const LOCALSTORAGE_BOARD_CUSTOM_FONT_SWITCH = 'board_custom_font_switch';

export class LocalStorageService implements IStorageService {
  loadOpenKeys(): Promise<string[]> {
    return new Promise(() => {
      const localStr = localStorage.getItem(LOCALSTORAGE_MENU_OPEN_KEYS);
      return localStr ? JSON.parse(localStr) : [];
    });
  }

  saveOpenKeys(openKeys: string[]): void {
    const localStr = JSON.stringify(openKeys);
    localStorage.setItem(LOCALSTORAGE_MENU_OPEN_KEYS, localStr);
  }

  saveCurrentFileId(fileId: string | undefined | null): void {
    localStorage.setItem(LOCALSTORAGE_CURRENT_FILE_ID, fileId || '');
  }

  loadCurrentFileId(): string | null {
    return localStorage.getItem(LOCALSTORAGE_CURRENT_FILE_ID);
  }

  loadBoardCustomFont(): string | null {
    return localStorage.getItem(LOCALSTORAGE_BOARD_CUSTOM_FONTS);
  }

  saveBoardCustomFont(fontName: string | null): void {
    fontName && localStorage.setItem(LOCALSTORAGE_BOARD_CUSTOM_FONTS, fontName);
  }

  addBoardCustomFont(fontFamilyName: string): void {
    const customFonts = localStorage.getItem(LOCALSTORAGE_BOARD_CUSTOM_FONTS);
    const arr = customFonts?.split(',');

    if (arr?.includes(fontFamilyName)) return;

    const newCustomFonts = arr ? arr.concat(fontFamilyName).join(',') : fontFamilyName;
    localStorage.setItem(LOCALSTORAGE_BOARD_CUSTOM_FONTS, newCustomFonts);
  }

  saveLangCode(langCode: string): void {
    localStorage.setItem(LOCALSTORAGE_LANG_CODE, langCode);
  }

  loadLangCode(): string | null {
    return localStorage.getItem(LOCALSTORAGE_LANG_CODE);
  }

  loadBoardCustomFontSwitch(): string | null {
    return localStorage.getItem(LOCALSTORAGE_BOARD_CUSTOM_FONT_SWITCH);
  }

  saveBoardCustomFontSwitch(value: boolean): void {
    localStorage.setItem(LOCALSTORAGE_BOARD_CUSTOM_FONT_SWITCH, value + '');
  }
}

export const localStorageService: LocalStorageService = new LocalStorageService();
