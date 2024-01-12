// Metrics APIs
import { NextRequest } from 'next/server';
import { fetchFromWA, newResponse } from '../../common';

interface Params {
  params: {
    target: string;
    target2: string;
  };
}

export async function GET(request: NextRequest, { params }: Params) {
  const searchParams = request.nextUrl.searchParams;
  const queryString = searchParams.toString();
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}${
      queryString ? `?${queryString}` : ''
    }`
  );
  return new Response(response.body, {
    headers: { 'content-type': 'application/json' },
  });
}

export async function UPDATE(request: Request, { params }: Params) {
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}`,
    {
      method: 'PUT',
      body: JSON.stringify(request.body),
    }
  );
  return newResponse(response.body);
}

export async function DELETE(request: Request, { params }: Params) {
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}`,
    {
      method: 'DELETE',
    }
  );
  return newResponse(response.body);
}
