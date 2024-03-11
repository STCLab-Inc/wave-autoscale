'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import { useInfo } from '@/services/info';
import YAMLEditor from '../common/yaml-editor';

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
