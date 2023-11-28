'use client';

import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import { usePathname } from 'next/navigation';

import Menu from './menu';

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();

  const [windowWidth, setWindowWidth] = useState<number>(0);

  useEffect(() => {
    const handleResize = () => {
      setWindowWidth(window.innerWidth);
    };

    setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  const [menuFlag, setMenuFlag] = useState<boolean>(false);

  useEffect(() => {
    if (windowWidth >= 880) {
      setMenuFlag(false);
    }
  }, [windowWidth]);

  useEffect(() => {
    setMenuFlag(false);
  }, [pathname]);

  const menuRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const clickOutside = (event: any) => {
      if (
        menuFlag &&
        menuRef.current &&
        !menuRef.current?.contains(event.target)
      ) {
        setMenuFlag(false);
      }
    };
    document.addEventListener('mousedown', clickOutside);
    return () => {
      document.removeEventListener('mousedown', clickOutside);
    };
  }, [menuFlag]);

  return (
    <div className="h-screen w-screen min-w-screen-md overflow-auto">
      <nav
        className="navbar relative h-14 w-full p-0 text-neutral-content"
        ref={menuRef}
      >
        <div className="flex h-full w-full items-center px-8">
          <div className="flex h-full w-full items-center justify-between">
            <figure
              onClick={() => router.push('/app')}
              className="cursor-pointer"
            >
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
            <div className="ml-10 flex h-full w-full items-center">
              <Menu
                windowWidth={windowWidth}
                menuFlag={menuFlag}
                setMenuFlag={setMenuFlag}
                responsive={true}
              />
            </div>
          </div>
        </div>
        {menuFlag ? (
          <div className="absolute top-16 z-30 flex w-full flex-col items-center overflow-y-auto border-b border-gray-200 bg-base-100 px-8 py-8">
            <Menu
              windowWidth={windowWidth}
              menuFlag={menuFlag}
              setMenuFlag={setMenuFlag}
              responsive={false}
            />
          </div>
        ) : null}
      </nav>
      <main className="wa-main">{children}</main>
    </div>
  );
}
