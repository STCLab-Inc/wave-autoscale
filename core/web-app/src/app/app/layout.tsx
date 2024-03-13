'use client';

import React from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { getQueryClient } from '@/infra/query-client';
import Sidebar from './sidebar';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

const queryClient = getQueryClient();

// Override console.error
// This is a hack to suppress the warning about missing defaultProps in recharts library as of version 2.12
// @link https://github.com/recharts/recharts/issues/3615
const error = console.error;
console.error = (...args: any) => {
  if (/defaultProps/.test(args[0])) return;
  error(...args);
};

export default function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="flex w-screen min-w-screen-xl flex-row justify-start overflow-auto pl-64">
        <Sidebar />
        <main className="wa-main min-h-screen flex-1 bg-gray-75">{children}</main>
      </div>
      <ToastContainer />
    </QueryClientProvider>
  );
}
