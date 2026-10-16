# S00: 확대 도구 화면의 설치 버튼 접근

- 상태: implementing
- 분류: bug
- 우선순위: P0
- 요구사항: R04, R08, R13
- 담당/수정 범위: 주 담당; crates/app/src/accessible.rs, crates/app/src/accessible_tools.rs, 이 문서. diagnostic_cache·diagnostic_inputs는 독립 /tmp 코드·시험 후보 담당.
- 선행 작업: S00-tools-action-first-polish-20261012

## 사용자 결과

800×600의 200% 배율에서도 도구 화면을 스크롤해 설치·상세·목록에 접근하고 키보드 포커스의 설치 버튼 전체를 확인한다.

## 계약과 범위

실제 current-ui-tools-action-first-20261012 캡처 세 탭 모두 설치 버튼이 footer 뒤로 가려졌다. 원 native lifecycle과 source pins는 통과했으나 UI 인수는 FAIL이다. 전체 본문을 단일 ScrollArea로 묶고 중첩 inventory 스크롤을 제거한다. worker·소유권·설치 gate·의존성·schema는 유지한다. 가짜 IntMax clip은 화면 공개 증거로 수용하지 않는다.

## 인수 조건

실제 Candidate 세 탭의 설치 버튼 focus 후 전체 버튼과 글자가 원 paint clip 및 footer 위에 들어간다. 실제 가상 화면 확대 캡처에서도 확인한다. 한 번 공개 후 사용자 wheel을 매 frame 되돌리지 않는다. 전체 검사·원 worker stop ACK와 실패 원본을 보존한다.

## 검증과 handoff

주 담당이 200% 세 캡처와 150% Deno 비교를 직접 확인했다. 실제 Context RED를 먼저 추가하며 아직 수정 후 인수는 아니다. native 전체 실행은 자연 종료0·원26pins/source전후 보존을 독립 확인했다. UI FAIL 기록은 root-tools-200-pixel-failure.json과 root-raw-and-ui-verdict.json에 보존했다.

수정 전 actual Context RED: 첫 실행은 시험 mutable borrow 컴파일 오류(exit101)였고 교정 후 재실행했다. corrected 실행은 실제 시험1개 실패: 설치 caption paint0으로 body 밖 숨김을 검출했다. 원 worker stop ACK·Job 자연 종료/정리를 확인했다. 주 담당은 본문 single ScrollArea와 설치 gained_focus 일회 공개 후보를 통합한다. GREEN 및 실제 확대 재캡처는 아직이다.

첫 GREEN은 yt-dlp 통과 후 Deno 전체 버튼이 bodyclip203을 넘어212.5까지 내려가 실패했다. 탭 변경 시 동일 설치 ID의 포커스가 유지되므로 gained_focus만으로 재공개되지 않았다. 주 담당은 tool_changed && install.has_focus인 변경 순간도 한 번 공개하도록 교정했다. corrected GREEN은 실제 시험1개 PASS/0FAIL: 세 탭 모두 버튼[133,163]–[242,195], glyph[145.5,173.2]–[228.5,186.2]가 원clip[0,102.5]–[388,203] 및 footer216 위에 들어갔다. 원 worker ACK·Job 자연 종료0 확인. 물리 pointer·사용자 wheel 및 실제 GUI 재캡처는 아직이다.

최신 current-ui-tools-scroll-checkbox-20261012 수집은 자연 종료0이며 독립 reviewer도 exit0이다. 141개 캡처(기본120·도구21), 현재 당시 소스·실행 파일 pins와 두 private cleanup을 대조했다. root가 200% 세 도구의 실제 after PNG를 직접 보았으며 설치 버튼 전체가 footer 위에 보였다. 이는 Light·포커스 후 버튼 표시 인수다. 사용자 wheel, 모든 Dark 도구 및 물리 pointer 인수로 확대하지 않는다. 상세 증거는 root-independent-raw-review.json 및 root-two-tools-pixel-review.json, root-additional-three-pixel-review.json이다. 이후 네트워크 안내 변경으로 이 proof는 해당 이전 snapshot에 한정한다.
