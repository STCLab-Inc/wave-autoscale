import { ReadonlyHeaders } from 'next/dist/server/web/spec-extension/adapters/headers';

export function getPathname(headers: ReadonlyHeaders) {
  console.dir(headers);
  return headers.get('next-url');
}
