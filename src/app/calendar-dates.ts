export function calendarDateKey(date: Date): string {
  return `${date.getFullYear().toString().padStart(4, "0")}-${(date.getMonth() + 1).toString().padStart(2, "0")}-${date.getDate().toString().padStart(2, "0")}`;
}

export function parseCalendarDate(key: string): Date {
  const [year, month, day] = key.split("-").map(Number);
  // Noon avoids midnight DST transitions; no ISO UTC conversion of local dates.
  return new Date(year, month - 1, day, 12);
}

export function calendarMonthCells(year: number, month: number): (string | null)[] {
  const offset = (new Date(year, month - 1, 1, 12).getDay() + 6) % 7;
  const days = new Date(year, month, 0, 12).getDate();
  const length = Math.ceil((offset + days) / 7) * 7;
  return Array.from({ length }, (_, index) => index < offset || index >= offset + days
    ? null : `${year.toString().padStart(4, "0")}-${month.toString().padStart(2, "0")}-${(index - offset + 1).toString().padStart(2, "0")}`);
}
