# S05: 자막 typed 옵션과 실행 계획

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R07, R05
- 담당/수정 범위: library_identity 에이전트; crates/core/src/options.rs, 승인된 신규 scripts/subtitle_smoke.py, 본 문서, docs/validation/subtitle-options.md
- 선행 작업: S05 advanced options; 앱 입력과 자막 artifact publication은 주 담당 후속

## 사용자 결과

한국어/영어/둘 다의 유한 선택과 별도 자동 생성 자막 허용으로 안전한 실행 계획을 만든다. 원본 언어 metadata 조회와 구간 자막 timestamp 조정은 추정하지 않는다.

## 계약과 범위

DownloadOptions.subtitles는 Option<SubtitleOptions>이며 기존 저장 DTO v1에 필드가 없어도 None이다. None 직렬화와 argv는 기존과 동일하다. 자막은 sidecar로 저장하며 변환·embedding은 추가하지 않는다. 구간+자막은 잘못된 시간축을 방지하기 위해 현재 거절한다. raw subtitle flag override는 advanced 정책으로 거절한다. schema/Cargo/engine/UI는 수정하지 않는다. Git 저장소 부재를 유지하며 .git을 재구성하지 않는다.

## 인수 조건

- typed finite language, explicit auto-generated, vtt/best argv가 하나씩만 나온다.
- 도움말에 설명·예·기본값·비용·충돌·버전 검증 상태를 제공한다.
- legacy 무자막 저장 DTO/argv 동일, 손상·unknown enum·reserved raw 옵션은 거절한다.
- 기존 yt-dlp로 loopback synthetic VTT만 시험하고 실제 영상·인증을 사용하지 않는다.
- artifact 목록/최종 publication 및 UI 미연결을 완료라고 보고하지 않는다.

## 검증과 handoff

core 20개 테스트와 clippy 통과. 기존 yt-dlp 2026.09.27.232945의 loopback 합성 VTT smoke 2개 조합 통과. [검증 기록](../validation/subtitle-options.md)에 실제 범위와 후속 publication 계약을 기록했다. 전체 통합과 미디어+sidecar 공개는 주 담당 조정, 원본 언어와 구간 자막 지원은 미완료다.
