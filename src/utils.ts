export const dateToTimestamp = (date: Date): number =>
  Math.floor(date.getTime() / 1_000);
