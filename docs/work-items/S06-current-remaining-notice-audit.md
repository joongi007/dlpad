# S06: 현재 Windows 후보의 남은 notice 범위 감사

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root 본문·mapping/공지문 채택·독립 검증; diagnostic_cache /tmp 원cache/lock/tree 읽기 감사만
- 선행 작업: 개발ZIP wholelock 누락43개 및 Windows feature normal/build 범위 실증

## 사용자 결과

현재 Windows 후보에 필요한 notice와 다른OS/선택feature의 lock superset 항목을 정확한 근거로 구분한다.

## 계약과 범위

원 Cargo.lock/cache crate·VCS·license bytes와 실제 feature/tree 증거를 읽는다. 기존28 source mapping·Winapi payload·폰트 원자료를 보존한다. 법적승인/최종binary reachability/전OS완비를 이름이나 metadata만으로 확정하지 않는다. Cargo/lock/schema 변경은 없고 하위는 /tmp만 작성한다. 실제 필수 누락으로 확인된 항목만 root가 추가자료를 채택한다.

## 인수 조건

현재43 inventory와 원lock·feature/tree의 정확 SHA 일치, 각 actual mapping의 source/notice byte 근거, 원cache 무변경, 현재후보/과거CPU snapshot 범위 구분. 전체legal/release_ready는 독립게이트를 유지한다.

## 검증과 handoff

읽기 감사 착수. 현재 native 하드링크 결함 회귀/비용 시험과 독립 수행하며 아직 신규 mapping/ZIP 완료가 아니다.


Root가3후보원crate checksum/현재lock/VCS 및원cache67members 전체byte·grant2원문SHA를다시대조했고scope읽기감사는완료했다. 실제 누락43을40으로줄였다고표시하지않는다. 해당3개는기존Windows GNU CPU normal/build tree에서미포함인otherOS항목이므로Windows 우선범위에서는production mapping에채택하지않고새ZIP도만들지않았다. exact초안/grant/원근거는 windows-remaining-notice-audit-20261005-continue에보존했다. pathpatch AT-SPI 공지는Linux후속별도정책이며전체legal/최종linkedclosure/release_ready는미완료다.
