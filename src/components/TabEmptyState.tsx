import React from 'react'
import { Button, Group, Text } from '@mantine/core'
import {
  IconArrowRight,
  IconCamera,
  IconCertificate,
  IconCheck,
  IconFileCode,
  IconSparkles,
} from '@tabler/icons-react'
import type { ImageMetadataReport } from '@/types/metadata'
import classes from './TabEmptyState.module.css'

interface TabEmptyStateProps {
  icon: React.ReactNode
  title: string
  description: string
  currentTab: string
  report?: ImageMetadataReport
  onNavigateTab?: (tab: string) => void
}

export const TabEmptyState: React.FC<TabEmptyStateProps> = ({
  icon,
  title,
  description,
  currentTab,
  report,
  onNavigateTab,
}) => {
  // Check which other categories have data
  const hasAi =
    currentTab !== 'ai' &&
    report?.ai &&
    (!!report.ai.prompt ||
      !!report.ai.platform ||
      !!report.ai.comfy_workflow_json)

  const hasExif =
    currentTab !== 'exif' && report?.exif && report.exif.all_tags.length > 0

  const hasC2pa = currentTab !== 'c2pa' && report?.c2pa && report.c2pa.has_c2pa

  const hasRaw =
    currentTab !== 'raw' &&
    report?.raw_chunks_found &&
    report.raw_chunks_found.length > 0

  const hasAnyOtherData = hasAi || hasExif || hasC2pa || hasRaw

  return (
    <div className={classes.emptyContainer}>
      <div className={classes.iconOrb}>{icon}</div>
      <Text className={classes.emptyTitle}>{title}</Text>
      <Text className={classes.emptyDesc}>{description}</Text>

      {hasAnyOtherData && onNavigateTab && (
        <div className={classes.switchPromptBox}>
          <Text size="xs" fw={700} c="dimmed" tt="uppercase" mb="xs">
            Detected metadata in other categories:
          </Text>
          <Group gap="xs" justify="center" wrap="wrap">
            {hasC2pa && (
              <Button
                size="xs"
                variant="light"
                color="teal"
                leftSection={<IconCertificate size={14} />}
                rightSection={<IconArrowRight size={12} />}
                onClick={() => onNavigateTab('c2pa')}
              >
                C2PA Credentials (Verified)
              </Button>
            )}

            {hasExif && (
              <Button
                size="xs"
                variant="light"
                color="indigo"
                leftSection={<IconCamera size={14} />}
                rightSection={<IconArrowRight size={12} />}
                onClick={() => onNavigateTab('exif')}
              >
                EXIF & GPS ({report?.exif?.all_tags.length} tags)
              </Button>
            )}

            {hasAi && (
              <Button
                size="xs"
                variant="light"
                color="cyan"
                leftSection={<IconSparkles size={14} />}
                rightSection={<IconArrowRight size={12} />}
                onClick={() => onNavigateTab('ai')}
              >
                AI Generation Prompts
              </Button>
            )}

            {hasRaw && (
              <Button
                size="xs"
                variant="light"
                color="gray"
                leftSection={<IconFileCode size={14} />}
                rightSection={<IconArrowRight size={12} />}
                onClick={() => onNavigateTab('raw')}
              >
                Raw Chunks ({report?.raw_chunks_found.length})
              </Button>
            )}
          </Group>
        </div>
      )}

      {!hasAnyOtherData && report && (
        <div className={classes.cleanFileNotice}>
          <IconCheck size={16} color="#34d399" />
          <Text size="xs" c="teal" fw={600}>
            Clean Image: No sensitive tracking or prompt metadata detected in
            this file.
          </Text>
        </div>
      )}
    </div>
  )
}
