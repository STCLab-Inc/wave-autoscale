// Metrics APIs
import { NextRequest } from 'next/server';
import { fetchFromWA, newResponse } from '../../../common';

interface Params {
  params: {
    target: string;
    target2: string;
    target3: string;
  };
}

export async function GET(request: NextRequest, { params }: Params) {
  const searchParams = request.nextUrl.searchParams;
  const queryString = searchParams.toString();
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}/${params.target3}${
      queryString ? `?${queryString}` : ''
    }`
  );
  return new Response(response.body, {
    headers: { 'content-type': 'application/json' },
  });
}

export async function POST(request: Request, { params }: Params) {
  const body = await request.text();
  console.log('body', body);
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}/${params.target3}`,
    {
      method: 'POST',
      body,
      headers: {
        // Expect that  all POST requests to the API will be JSON
        'Content-Type': 'application/json',
      },
    }
  );
  return newResponse(response.body);
}

export async function PUT(request: Request, { params }: Params) {
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}/${params.target3}`,
    {
      method: 'PUT',
      body: JSON.stringify(request.body),
    }
  );
  return newResponse(response.body);
}

export async function DELETE(request: Request, { params }: Params) {
  const response = await fetchFromWA(
    `/api/${params.target}/${params.target2}/${params.target3}`,
    {
      method: 'DELETE',
    }
  );
  return newResponse(response.body);
}
