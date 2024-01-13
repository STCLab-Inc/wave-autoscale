'use client';

import React from 'react';
import ContentHeader from '../common/content-header';
import TEMPLATES, { TemplateSection } from '../../../data/wa-templates';
import { TemplateItem } from '../common/template-item';

const ALL_TEMPLATES = [...TEMPLATES.SCALING_COMPONENTS, ...TEMPLATES.METRICS];

function TemplateSection({
  templateSection,
}: {
  templateSection: TemplateSection;
}) {
  return (
    <div className="flex flex-col px-10 py-8">
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
      <ContentHeader
        type="OUTER"
        title="Add Plan"
        right={<div>Add a plan with a variety of templates </div>}
      />
      {/* Shortcuts for Templates */}
      <div className="flex flex-row items-center space-x-2 px-10 py-8">
        <span className="font-bold">Scaling Components</span>
        {TEMPLATES.SCALING_COMPONENTS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-link btn-sm btn"
          >
            {templateSection.title}
          </a>
        ))}
        <span className="!ml-12 font-bold">Metrics</span>
        {TEMPLATES.METRICS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-link btn-sm btn"
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
