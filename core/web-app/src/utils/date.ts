import dayjs, { Dayjs } from 'dayjs';

// Regex to match unix timestamp
const unixTimestampRegex = /^\d{10}$/;
const unixTimestampMsRegex = /^\d{13}$/;
const unixTimestampMsWithPointRegex = /^\d{10}\.\d{3}$/;

export function parseDateToDayjs(date: Date | string | number): Dayjs {
  // If date is Date type
  if (date instanceof Date) {
    return dayjs(date);
  }
  // If date is string or number type
  // If date is unix timestamp
  if (
    unixTimestampRegex.test(date.toString()) ||
    unixTimestampMsWithPointRegex.test(date.toString())
  ) {
    return dayjs.unix(Number(date));
  } else if (unixTimestampMsRegex.test(date.toString())) {
    return dayjs(Number(date));
  }
  // Else, parse date with dayjs
  return dayjs(date);
}
