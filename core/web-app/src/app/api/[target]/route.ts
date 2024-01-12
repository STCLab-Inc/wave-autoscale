// Metrics APIs
import { NextRequest } from 'next/server';
import { fetchFromWA, newResponse } from '../common';

interface Params {
  params: { target: string };
}

export async function GET(request: NextRequest, { params }: Params) {
  const searchParams = request.nextUrl.searchParams;
  const queryString = searchParams.toString();
  const response = await fetchFromWA(`/api/${params.target}${queryString ? `?${queryString}` : '' }`);
  return new Response(response.body, {
    headers: { 'content-type': 'application/json' },
  });
}

export async function POST(request: Request, { params }: Params) {
  const response = await fetchFromWA(`/api/${params.target}`, {
    method: 'POST',
    body: JSON.stringify(request.body),
  });
  return newResponse(response.body);
}
