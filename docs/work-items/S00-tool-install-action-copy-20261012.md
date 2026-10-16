# S00: 선택 도구의 명확한 설치 동작

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 catalog.rs·accessible_tools.rs·accessible.rs 및 Windows C# 관측기3개와 이 문서
- 선행 작업: 실제 도구 화면의 기술 용어 중심 버튼 검토

## 사용자 결과

선택 도구에 따라 yt-dlp 설치, Deno 설치, FFmpeg·ffprobe 설치를 표시한다.

## 계약과 범위

중앙 catalog를 사용하며 설치 worker·활성화·중단 정책과32px 버튼을 유지한다. 기존 iced 호스트의 문구는 유지한다. native 관측기는 선택 도구별 정확한 문구를 검사한다. 새 의존성·schema 변경 없음.

## 인수 조건

세 도구의 버튼 문구·focus·glyph·disabled gate를 실제 egui와Windows Light/Dark 및200%에서 확인한다. 기존 설치 실패 테스트와전체검사·문서검사를 통과한다. 과거script의공급disabled가정을현재검증으로쓰지않는다.

## 검증과 handoff

관측기 순수테스트31/34 통과. 제품 통합 이후 whole/native 검증은 진행 중이다.

## 최신 scoped 검증 — 2026-10-12

Windows corrected whole1143/0/30 및 ordinary149 raw/provider 인수와 Dark200 focused11을 완료했다. Dark200 첫Light→선택Dark/재시작Dark, readonlySQLiteDark/integrityok/SHA전후, 세도구 설치영역 직접PNG 검토를 확인했다. 증거: [whole](../validation/evidence/windows-msvc-tool-action-navigation-corrected-whole-20261012/root-review.json), [Dark200 provider/DB](../validation/evidence/current-ui-tools-dark200-focused-v2-20261012/root-independent-provider-sqlite-review.json), [세영역픽셀](../validation/evidence/current-ui-tools-dark200-focused-v2-20261012/root-three-tools-pixel-review.json). v1noop/alias검토실패 원본 보존. 전체contents/물리hover/실제설치/userUIUX 인수가 아니므로 전체작업완료로확대하지 않는다. 동일1143 소스의 최신 패키지 실제5파일 설치·재검증 및 현재27 native ID 대조를 완료했다. [설치 기록](S11-current-ui-tool-action-package-20261012.md). 후속 Library 수정은 이 ZIP에 포함되지 않는다.
