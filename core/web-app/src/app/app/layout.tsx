'use client';

import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import { usePathname } from 'next/navigation';
import Menu from './common/menu';

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();

  return (
    <div className="flex w-screen min-w-screen-md flex-row overflow-auto">
      <nav
        className="sidebar min-w-64 w-64 border-r border-wa-gray-500"
        // ref={menuRef}
      >
        {/* Logo */}
        <figure
          onClick={() => router.push('/app')}
          className="h-14 w-full border-b border-wa-gray-200 bg-white px-4 py-3"
        >
          <Image
            src="/assets/sidebar/logo.svg"
            alt="Logo"
            priority={true}
            width={165}
            height={32}
            // style={{
            //   minWidth: '2.5rem',
            //   maxWidth: '2.5rem',
            // }}
          />
        </figure>

        {/* Menu */}
        <Menu />
      </nav>
      <main className="wa-main min-h-screen flex-1">{children}</main>
    </div>
  );
}
