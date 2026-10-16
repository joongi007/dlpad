# S00: 도구 설치 화면의 정보 위계 개선

- 상태: implementing
- 분류: UX
- 우선순위: P0
- 요구사항: R04, R08, R13
- 담당/수정 범위: 주 담당 통합; crates/app/src/catalog.rs, crates/app/src/accessible_tools.rs, 이 문서. diagnostic_inputs는 /tmp 후보만 작성.
- 선행 작업: current-ui-download-tool-recovery-20261012 도구 화면 캡처 검토

## 사용자 결과

선택한 도구의 역할, 공급 출처, 다운로드·설치 크기와 설치 행동을 먼저 읽는다. 기술 검증 정보와 플랫폼 식별자는 펼침 영역에서 확인한다.

## 계약과 범위

PRODUCT·DESIGN 및 GTMUX tokens.css의 정보 위계와 일관된 컴포넌트를 따른다. Rust native·중앙 catalog·기존 worker/설치 소유권을 유지한다. Cargo·schema·공급 trust 변경 없음. 공개 배포 준비 상태를 숨기거나 승인 완료로 표현하지 않는다.

## 인수 조건

yt-dlp·Deno·FFmpeg 각 역할과 출처가 정확하고 설치 전 다운로드 크기와 설치 파일 크기가 보인다. 기술 정보는 펼침에서 읽을 수 있고 설치·새로고침·중단의 기존 실행 조건이 유지된다. 800폭·150/200%에서 일반 및 펼침 상태의 실제 캡처를 검토한다. 전체 fmt/clippy/test/release와 원본·자연 종료 검증을 수행한다.

## 검증과 handoff

주 담당이 initial-tools-800.png를 직접 확인했다. 긴 HTTPS/SHA 설명과 target triple이 기본 화면에 노출되고 설치 행동이 같은 무게의 버튼 행에 묻힌다. 개선 후보 검토 중이며 변경 후 실행 검증은 아직이다.

주 담당 통합: 후보 source pins 및 전체 diff를 대조해 두 파일에 반영했다. 기존 설치 worker·실행 조건·공급 manifest·취소 신호는 유지했다. 설치 primary와 작업 중에만 보이는 중단 버튼을 사용한다. archive 설치가 자동 활성화된다고 표현하지 않도록 공통 문구를 교정했다. 새 의존성·schema 변경 및 추가 I/O는 없다. 전체 검사와 실제 화면 인수는 아직이다.
