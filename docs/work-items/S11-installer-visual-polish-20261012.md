# S11: 설치 화면 시각 정리

- 상태: preparation
- 분류: ui-quality
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: release_preparation 설치nativeUI 후보(/tmp), 주 담당 공유catalog·통합·검증·본 문서
- 선행 작업: 실제설치5PNG 직접감사(완료)

## 사용자 결과

설치 위치·선택 도구·진행 결과의 우선순위와 글꼴·간격을 앱의 DESIGN에 맞춰 읽기 쉽게 한다.

## 계약과 범위

Rust/Win32 native 유지. 설치/중단/재검증/제거 정책·소유권·worker·기존폴더보존 불변. Cargo/schema 변경은 주 담당만 조정.

## 인수 조건

실제private 설치PNG에서label/controls 정렬과font 가독성·긴경로표시확인, nativebrowse와optionaltools 및existing failurepaths 유지. 다시실제5file설치/reverify/pins/IDs 검토. 사용자UI승인과unsignedpublicrelease조건은별도.

## 검증과 handoff

기존density5PNG 주담당직접검토에서단순한계층/긴경로표현을개선대상으로기록했다. 담당이원nativeUI source를읽고후보준비중이며아직구현완료아니다.

기존networkwhole4검사/currentproof검토와sourcefreeze종료후주담당이원SHA대조하여후보통합. 체크박스28px최소유지·버튼32px·24margin·journal48px읽기전용multiline/autoverticalscroll로긴경로표현개선. 설치worker/ownership/ID불변. compile/native검증전.

## 읽기 전용 설치 기록 경로 실제 검증

windows-msvc-installer-pressed-feedback-final-20261012-install-text-pattern에서 실제 지원되는 TextPattern 전체 원문을 bounded Win32 text와 initial/wrapped/restored 세 번 정확 대조했다. 원본 source 보존, 자연 app exit0·Job0·가상 화면 종료·사용자 화면 유지와 5파일 설치·재검증을 독립 확인했다. 순수 실패 경로7개도 통과했다. 주 담당이 long-journal-wrap-selection.png를 직접 확인했다. 클립보드 복사·물리 IME·전체 시각 인수는 미검증이며 시스템 글꼴 및 설명 밀도 개선은 남는다. 이전 ValuePattern 실패는 보존한다.
