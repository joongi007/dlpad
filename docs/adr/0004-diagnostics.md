# ADR-0004: 로컬 진단 자료와 선택적 AI 분석

상태: P0 로컬 진단 방향은 Accepted(2026-10-16 Root가 구현·실험 근거로 채택), P1 알려진 문제와 P2 API 연결은 Proposed. 이는 전체 S10·Windows 출시 인수 또는 사용자 최종 UI 승인 상태가 아니다. 근거: 외부 yt-dlp 버그가 발생해도 사용자가 원인과 다음 행동을 알 수 있어야 한다는 추가 요구(R13).

P0 결정: Attempt 증거를 정제한 진단 보고서를 기본 제공한다.

P1/P2 제안: 알려진 문제 규칙은 근거·버전·확인 시각이 있는 데이터로 관리한다. AI는 기본 설치·실행 의존성이 아니며 복사한 자료로 외부에서 분석할 수 있다. 앱 내 API 연결은 후속 선택 기능이다.

대안: 모든 실패를 AI에 자동 전송하면 비용·개인정보·오진·네트워크 의존성이 커진다. 앱이 yt-dlp 소스를 자동 패치하면 재현성과 도구 공급망이 복잡해진다. 둘 다 기본 동작으로 채택하지 않는다.

결과: 로컬 분류·AI 추정·검증된 수정 배포를 구별해야 한다. 진단 보고서는 작업 실행 권한을 갖지 않는다. Issue가 closed라는 이유로 해결됨을 표시하지 않는다. 세부 계약은 [진단 설계](../DIAGNOSTICS.md)를 따른다.

## 2026-09-30 구현 증거

로컬 evidence 정제, bounded Markdown/JSON, 반복 URL 대체값, MPD range/속성 부재 보존, DTD/entity 거절, create-new export와 fixture 테스트를 구현했다. 개발 UI에서 실패 Attempt의 stage/tool version 보고서를 보고 복사/저장할 수 있다. API/외부 검색은 호출하지 않는다.

당시에는 개별 로그 편집·첨부 선택의 UI와 실제 OS clipboard/export 검수가 남아 있었다. 이 단락은 2026-09-30 시점 기록이며 현재 미구현 목록이 아니다. P0 출시에 S10을 포함하는 계약은 유지한다.


## 2026-10-16 P0 방향 채택과 현재 인수 범위

Root는 사용자가 허용한 실험 근거 기반 아키텍처 결정으로, 오프라인 Attempt 보고서·정제된 Markdown/JSON·사용자 편집/제외·로컬 MPD 구조 첨부·bounded 미리보기·create-new 내보내기를 P0 방향으로 채택한다. 자동 외부 전송·원본 비밀 fallback·원격 패치는 도입하지 않는다. 후보 라이브러리·schema·공유 interface를 새로 바꾸는 결정은 아니다.

편집·제외·첨부·상한·CloseWait에는 기존 실제 Windows native11case와 현재 whole33의24개 매핑행(중복 포함) 및 별도 feature전용2행이 있다. 동일한 core와 변경된 UI/adapter를 구분해 [근거를 연결했다](../validation/evidence/current33-p0-closeout-root-20261016/S10-MAP.md). 이전 native 결과를 현재33 전체 재실행으로 확대하지 않는다.

v33은100/200%에서 JSON971B 미리보기·create-new 저장·같은 경로 거절·원파일/실패DB 보존을 실제 확인했고, 읽을 수 있는 미리보기 높이도 개선했다. [원 검토](../validation/evidence/current33-s10-preview-composite-root-20261016/root-review.json). 설계 채택은 실제 Windows OS clipboard의 미완료를 해소하지 않는다. 독립 station 공급 제약과 사용자 clipboard fallback 금지를 유지하며 별도 격리 환경의 OS readback을 남긴다.

P1 카탈로그·이슈 조회 및 P2 API 연결에는 착수하지 않는다. 전체 P0·최종 UI/OS 인수는 [현재 잔여](../validation/P0-current-20261016.md)를 따른다. [이번 결정 기록](../work-items/S10-current33-adr-reconciliation-20261016.md).
