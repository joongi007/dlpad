# S11: v28 UI 검수 후보 패키지 준비

- 상태: scoped 검증 완료 / 서명 없는 개발 검수 후보
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R10, R13; S11 Windows 후보 검수
- 담당/수정 범위: search_reveal, B/ui-feedback-v28-package-v1 scratch만. Root가 docs/work-items 채택·실행·통합을 담당한다.
- 선행 작업: exact whole28 fmt/clippy/test/release 모두 성공, sibling root-review.json이 actual source/raw43/3PE를 독립 대조한다.

## 사용자 결과

통합된 v28 UI를 사용자가 설치 또는 portable 앱으로 검수할 수 있도록 새 배포 후보를 만든다. 실제 설치·portable ZIP을 생성하고 아래 범위에서 검증했다.

## 계약과 범위

v27 원 helper/ZIP/generated/evidence를 보존한다. v28 전체 7단계 argv와 provenance/CRC/SHA/notice/current source/create-only 검사를 유지한다. 새 dependency, Cargo/lock/schema, 제품 소스, 공개 업로드 변경은 없다. review 경로는 exact sibling root-review.json으로 강화했다.

## 인수 조건

Root가 실제 whole28 독립 검토 뒤 각 새 단계의 원로그·Job 종료·현재 source와3PE·전체 ZIP streaming CRC/SHA·selected notice를 검토한다. source 수·실행파일 SHA·test 합계는 actual proof에서 읽는다. graph 도메인이 retained258/43에서 달라지면 중단해 근거를 보존한다. 독립 최종 검토를 통과해야 새 ZIP을 최신으로 안내한다. release/legal/signature/clean OS/긴 경로 인수는 별개다.

## 검증과 handoff

준비 시 Python AST만 검사했다. package/native/Cargo 실행0. HANDOFF.md와 argv.json이 다음 명령의 권위 있는 목록이다. Root가 이 draft를 docs/work-items에 채택한다. 비용은 helper 텍스트 복사뿐이며 앱 런타임 의존성·worker·스키마 변경이 없다.

Root 실제 패키징 완료: whole43harness·graph선택258·원notice456, portable467 members 및 installer7 members 전체CRC/SHA를독립검산했습니다. portable18288695B SHA1f50f9fe34efc771ca63c4abb4511fcca0f802d35cb11cd6dcdd3e6793a7754d, installer17238199B SHA40b085a1d62ee005fc9844ef85aea26f8dbf26c7e8922d7cb5e564aa190b16e1. 실제2installer+1CPU앱·새/기존journal5+5·최종payload5·33pins·자연종료0/Job0·Root직접3PNG를확인했습니다. 공개릴리스·서명·전체legal·물리cleanOS·긴경로의현재버전인수는미완료이며자동upgrade는미지원입니다. README/실행안내를28로갱신하고이전안내는별도보존했습니다.
