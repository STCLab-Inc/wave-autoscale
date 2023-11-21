'use client';
import Image from 'next/image';
import { Inter } from 'next/font/google';
import classNames from 'classnames';
import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';

const inter = Inter({ subsets: ['latin'] });

export default function Home() {
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

  const paddingClass = windowWidth < 768 ? 'p-5' : 'p-10';

  const cardClass = classNames('card m-12 shadow-2xl mx-auto max-w-lg', {
    'sm:card-side': windowWidth < 640,
    'md:card-side': windowWidth >= 640 && windowWidth < 1024,
    'lg:card-side': windowWidth >= 1024,
  });

  const router = useRouter();

  const handleClick = () => {
    router.push('/app');
  };

  return !windowWidth ? null : (
    <div className="flex justify-center">
      <div
        className={`${cardClass} cursor-pointer transition-transform hover:scale-105`}
        onClick={handleClick}
      >
        <figure
          className={`bg-white p-5`}
          style={{ minWidth: '10rem', minHeight: '10rem' }}
        >
          <Image
            src="/assets/images/wave-autoscale_symbol.svg"
            alt="wave-autoscale_symbol.svg"
            priority={true}
            width={100}
            height={100}
            style={{ minWidth: '5rem', maxWidth: '5rem' }}
          />
        </figure>
        <figure className={`bg-black ${paddingClass}`}>
          <Image
            src="/assets/images/wave-autoscale_text.svg"
            alt="wave-autoscale_text.svg"
            priority={true}
            width={100}
            height={100}
          />
        </figure>
      </div>
    </div>
  );
}
