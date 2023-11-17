'use client';

import React from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';

import Menu from './menu';

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();

  const handleClick = () => {
    router.push('/app');
  };

  return (
    <div className="h-screen w-screen min-w-screen-md overflow-auto">
      <nav className="navbar h-14 w-full p-0 text-neutral-content">
        <div className="flex h-full w-full items-center pl-8 pr-8">
          <div className="flex h-full items-center justify-between">
            <figure onClick={handleClick} className="cursor-pointer">
              <Image
                src="/assets/images/wave-autoscale_symbol.svg"
                alt="wave-autoscale_symbol.svg"
                priority={true}
                width={36}
                height={32}
                style={{
                  minWidth: '2.5rem',
                  maxWidth: '2.5rem',
                }}
              />
            </figure>
            <div className="ml-10 flex h-full items-center">
              <Menu />
            </div>
          </div>
        </div>
      </nav>
      <main className="wa-main">{children}</main>
    </div>
  );
}
