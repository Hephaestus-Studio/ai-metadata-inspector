import React from 'react'
import { Badge, Group, Stack, Text } from '@mantine/core'
import {
  IconCertificate,
  IconCertificateOff,
  IconFingerprint,
} from '@tabler/icons-react'
import type { C2paReport, ImageMetadataReport } from '@/types/metadata'
import { TabEmptyState } from './TabEmptyState'
import classes from './C2paTab.module.css'

interface C2paTabProps {
  c2pa?: C2paReport
  report?: ImageMetadataReport
  onNavigateTab?: (tab: string) => void
}

export const C2paTab: React.FC<C2paTabProps> = ({
  c2pa,
  report,
  onNavigateTab,
}) => {
  if (!c2pa || !c2pa.has_c2pa) {
    return (
      <TabEmptyState
        icon={<IconCertificateOff size={28} stroke={1.5} />}
        title="No C2PA Content Credentials"
        description="This file does not contain Content Authenticity Initiative (CAI) cryptographic assertions, provenance signatures, or JUMBF containers."
        currentTab="c2pa"
        report={report}
        onNavigateTab={onNavigateTab}
      />
    )
  }

  return (
    <Stack gap="md">
      <Group justify="space-between" align="center">
        <Badge
          variant="gradient"
          gradient={{ from: 'teal', to: 'cyan' }}
          size="lg"
          leftSection={<IconCertificate size={16} />}
        >
          C2PA / Content Credentials Present
        </Badge>
        <Badge variant="outline" color="teal">
          {c2pa.box_count} JUMBF Box(es)
        </Badge>
      </Group>

      <div className={classes.card}>
        <Stack gap="xs">
          {c2pa.generator && (
            <div className={classes.itemRow}>
              <span className={classes.label}>Detected Platform:</span>
              <span className={classes.value}>{c2pa.generator}</span>
            </div>
          )}

          {c2pa.claim_generator && (
            <div className={classes.itemRow}>
              <span className={classes.label}>Claim Generator:</span>
              <span className={classes.value}>{c2pa.claim_generator}</span>
            </div>
          )}

          {c2pa.signature_issuer && (
            <div className={classes.itemRow}>
              <span className={classes.label}>Certificate Issuer:</span>
              <span className={classes.value}>{c2pa.signature_issuer}</span>
            </div>
          )}
        </Stack>
      </div>

      <Group gap="xs" c="dimmed">
        <IconFingerprint size={16} />
        <Text size="xs">
          {c2pa.raw_manifest_summary ||
            'Content Authenticity Initiative (CAI) digital assertions embedded in container.'}
        </Text>
      </Group>
    </Stack>
  )
}
