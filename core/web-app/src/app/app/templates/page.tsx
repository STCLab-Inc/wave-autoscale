'use client';

import React from 'react';
import ContentHeader from '../common/content-header';
import TEMPLATES, { TemplateSection } from '../../../data/wa-templates';
import { TemplateItem } from '../common/template-item';
import PageHeader from '../common/page-header';

const ALL_TEMPLATES = [...TEMPLATES.SCALING_COMPONENTS, ...TEMPLATES.METRICS];

function TemplateSection({
  templateSection,
}: {
  templateSection: TemplateSection;
}) {
  return (
    <div className="flex flex-col px-6 py-8">
      <h3 className="mb-4 text-lg font-semibold" id={templateSection.title}>
        {templateSection.title}
      </h3>
      <div className="grid grid-cols-3 gap-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
        {templateSection.templates.map((template) => (
          <TemplateItem
            key={template.title}
            template={template}
            isCard={true}
          />
        ))}
      </div>
    </div>
  );
}

export default function AddPlanPage() {
  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <PageHeader
        title="Templates"
        subtitle="Add a plan with a variety of templates"
      />
      {/* Shortcuts for Templates */}
      <div className="mx-6 flex h-14 flex-row items-center space-x-2 border-b border-wa-gray-200">
        <span className="font-medium">Scaling Components</span>
        {TEMPLATES.SCALING_COMPONENTS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-sm btn rounded-2xl"
          >
            {templateSection.title}
          </a>
        ))}
        <span className="!ml-12 font-medium">Metrics</span>
        {TEMPLATES.METRICS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-sm btn rounded-2xl"
          >
            {templateSection.title}
          </a>
        ))}
      </div>
      {/* Templates Grid */}
      {ALL_TEMPLATES.map((templateSection) => (
        <TemplateSection
          key={templateSection.title}
          templateSection={templateSection}
        />
      ))}
    </main>
  );
}
