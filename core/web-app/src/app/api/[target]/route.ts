// Metrics APIs
import { NextRequest } from 'next/server';
import { fetchFromWA, newResponse } from '../common';

interface Params {
  params: { target: string };
}

export async function GET(request: NextRequest, { params }: Params) {
  const searchParams = request.nextUrl.searchParams;
  const queryString = searchParams.toString();
  const response = await fetchFromWA(
    `/api/${params.target}${queryString ? `?${queryString}` : ''}`
  );
  return response;
}

export async function POST(request: Request, { params }: Params) {
  const body = await request.text();
  const response = await fetchFromWA(`/api/${params.target}`, {
    method: 'POST',
    body: body,
    headers: {
      // Expect that  all POST requests to the API will be JSON
      'Content-Type': 'application/json',
    },
  });
  return response;
}
