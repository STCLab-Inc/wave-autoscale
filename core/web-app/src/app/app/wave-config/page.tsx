'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import { useInfo } from '@/services/info';
import dynamic from 'next/dynamic';

// Dynamic imports (because of 'window' object)
const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

export default function WaveConfigPage() {
  const { data: info } = useInfo();
  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <PageHeader title="wave-config" subtitle="See wave-config.yaml" />
      <YAMLEditor value={info?.wave_config} readonly />
    </main>
  );
}
