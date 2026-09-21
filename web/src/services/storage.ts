export interface DepositNote {
  id: string;
  nullifierSecret: string; // hex
  secret: string; // hex
  noteSecret: string; // hex
  amount: number; // lamports
  commitment: string; // hex
  nullifierHash: string; // hex
  timestamp: number;
  used: boolean;
}

const STORAGE_KEY = "zkpool_deposits";

/**
 * Returns all deposit notes from localStorage.
 */
export function getDepositNotes(): DepositNote[] {
  try {
    const data = localStorage.getItem(STORAGE_KEY);
    if (!data) return [];
    return JSON.parse(data) as DepositNote[];
  } catch {
    return [];
  }
}

/**
 * Saves a new deposit note.
 */
export function saveDepositNote(note: DepositNote): DepositNote {
  const notes = getDepositNotes();
  notes.push(note);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(notes));
  return note;
}

/**
 * Returns a deposit note by nullifierHash.
 */
export function getDepositNoteByNullifier(nullifierHash: string): DepositNote | null {
  const notes = getDepositNotes();
  return notes.find((n) => n.nullifierHash === nullifierHash) || null;
}

/**
 * Returns a deposit note by id.
 */
export function getDepositNoteById(id: string): DepositNote | null {
  const notes = getDepositNotes();
  return notes.find((n) => n.id === id) || null;
}

/**
 * Marks a deposit note as used.
 */
export function markNoteAsUsed(nullifierHash: string): DepositNote | null {
  const notes = getDepositNotes();
  const index = notes.findIndex((n) => n.nullifierHash === nullifierHash);

  if (index === -1) return null;

  const existing = notes[index];
  if (!existing) return null;

  const updated: DepositNote = { ...existing, used: true };
  notes[index] = updated;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(notes));
  return updated;
}

/**
 * Removes a deposit note.
 */
export function removeDepositNote(nullifierHash: string): boolean {
  const notes = getDepositNotes();
  const filtered = notes.filter((n) => n.nullifierHash !== nullifierHash);

  if (filtered.length === notes.length) return false;

  localStorage.setItem(STORAGE_KEY, JSON.stringify(filtered));
  return true;
}

/**
 * Returns only unused deposit notes.
 */
export function getUnusedDepositNotes(): DepositNote[] {
  return getDepositNotes().filter((n) => !n.used);
}

/**
 * Clears all deposit notes.
 */
export function clearDepositNotes(): void {
  localStorage.removeItem(STORAGE_KEY);
}
