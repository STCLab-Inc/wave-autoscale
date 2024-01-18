'use client';

import React from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import { usePathname } from 'next/navigation';
import Menu from './common/menu';
import Link from 'next/link';
import { QueryClientProvider } from '@tanstack/react-query';
import { getQueryClient } from '@/infra/query-client';
import Sidebar from './sidebar';

const queryClient = getQueryClient();

export default function AppLayout({ children }: { children: React.ReactNode }) {

  return (
    <QueryClientProvider client={queryClient}>
      <div className="flex w-screen min-w-screen-xl flex-row justify-start overflow-auto pl-64">
        <Sidebar />
        <main className="wa-main min-h-screen flex-1">{children}</main>
      </div>
    </QueryClientProvider>
  );
}
