// Extend the fetch API to add some configuration

let API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;
if (!API_BASE_URL) {
  API_BASE_URL = 'http://localhost:3024';
}

export async function fetchFromWA(path: string, options?: any) {
  const url = `${API_BASE_URL}${path}`;
  const response = await fetch(url, options);
  return response;
}

export function newResponse(body: any, options?: any) {
  return new Response(body, {
    headers: { 'content-type': 'application/json' },
    ...options,
  });
}
