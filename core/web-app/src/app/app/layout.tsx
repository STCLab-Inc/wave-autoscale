'use client';

import React from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { getQueryClient } from '@/infra/query-client';
import Sidebar from './sidebar';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

const queryClient = getQueryClient();

export default function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="flex w-screen min-w-screen-xl flex-row justify-start overflow-auto pl-64">
        <Sidebar />
        <main className="wa-main min-h-screen flex-1">{children}</main>
      </div>
      <ToastContainer />
    </QueryClientProvider>
  );
}
