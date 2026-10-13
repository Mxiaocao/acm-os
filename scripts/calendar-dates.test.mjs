import assert from 'node:assert/strict';
import test from 'node:test';
import { calendarDateKey, calendarMonthCells, parseCalendarDate } from '../src/app/calendar-dates.ts';

test('calendar lays out leap February and month boundaries in Monday-first complete weeks', () => {
  const leap = calendarMonthCells(2024, 2);
  assert.equal(leap.filter(Boolean).length, 29);
  assert.equal(leap.length % 7, 0);
  assert.equal(leap[3], '2024-02-01');
  assert.ok(leap.includes('2024-02-29'));
  assert.equal(calendarMonthCells(2025, 2).filter(Boolean).length, 28);
  const august = calendarMonthCells(2026, 8);
  assert.equal(august.length, 42);
  assert.equal(august[5], '2026-08-01');
  const next = parseCalendarDate('2026-12-31');
  next.setDate(next.getDate() + 1);
  assert.equal(calendarDateKey(next), '2027-01-01');
  assert.equal(calendarDateKey(parseCalendarDate('2026-10-09')), '2026-10-09');
});
