import { exists, readTextFile, writeTextFile, remove, BaseDirectory } from "@tauri-apps/plugin-fs";

const SESSION_FILE = "session.json";
const BASE = BaseDirectory.AppLocalData;

export interface StoredSession {
  user_id: number;
  username: string;
  token: string;
}

export async function saveSession(session: StoredSession): Promise<void> {
  await writeTextFile(SESSION_FILE, JSON.stringify(session), { baseDir: BASE });
}

export async function loadSession(): Promise<StoredSession | null> {
  try {
    if (!await exists(SESSION_FILE, { baseDir: BASE })) return null;
    const raw = await readTextFile(SESSION_FILE, { baseDir: BASE });
    return JSON.parse(raw) as StoredSession;
  } catch {
    return null;
  }
}

export async function clearSession(): Promise<void> {
  try {
    await remove(SESSION_FILE, { baseDir: BASE });
  } catch {
    // file may not exist, that's fine
  }
}
