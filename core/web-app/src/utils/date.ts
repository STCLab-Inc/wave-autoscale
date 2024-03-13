import dayjs, { Dayjs } from 'dayjs';
import { decodeTime } from 'ulid';

// Regex to match unix timestamp
const unixTimestampRegex = /^\d{10}$/;
const unixTimestampMsRegex = /^\d{13}$/;
const unixTimestampMsWithPointRegex = /^\d{10}\.\d{3}$/;

export function parseDateToDayjs(
  date: Date | string | number
): Dayjs | undefined {
  if (!date) {
    return;
  }
  // If date is Date type
  if (date instanceof Date) {
    return dayjs(date);
  }
  // Mostly, Wave Autoscale uses unix timestamp in milliseconds
  if (unixTimestampMsRegex.test(date.toString())) {
    return dayjs(Number(date));
  } else if (
    // If date is string or number type
    // If date is unix timestamp
    unixTimestampRegex.test(date.toString()) ||
    unixTimestampMsWithPointRegex.test(date.toString())
  ) {
    return dayjs.unix(Number(date));
  }
  // Else, parse date with dayjs
  else return dayjs(date);
}

export function parseUlidToDayjs(ulid: string): Dayjs | undefined {
  return parseDateToDayjs(decodeTime(ulid));
}

export function parseUlidToUnixTimestamp(ulid: string): number | undefined {
  return decodeTime(ulid);
}
