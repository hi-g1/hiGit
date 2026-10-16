import { describe, expect, it } from "vitest";
import type { RewritePlanView } from "../../lib/api";
import {
  canJoin,
  changed,
  fromPlan,
  groups,
  joinWithBelow,
  move,
  setMessage,
  split,
  status,
  summary,
  toggleDrop,
  toInput,
  type Card,
} from "./model";

/** 새 것부터 다섯 장: e(맨 위) d c b a(맨 아래). */
function plan(n = 5): RewritePlanView {
  const names = "abcdefghijklmnopqrstuvwxyz".split("");
  return {
    cards: Array.from({ length: n }, (_, i) => {
      const id = names[n - 1 - i]!;
      return { oid: id.repeat(40), shortOid: id.repeat(7), summary: id, message: `${id}\n`, author: "지원", time: i };
    }),
    blocked: null,
    staleLabels: 0,
  };
}
const order = (cards: Card[]) => cards.map((c) => c.summary).join("");

describe("카드 모델", () => {
  it("계획에서 카드, 처음은 바뀐 것 없음", () => {
    const cards = fromPlan(plan());
    expect(order(cards)).toBe("edcba");
    expect(changed(cards, cards)).toBe(false);
    expect(summary(cards)).toEqual({ after: 5, dropped: 0, joined: 0 });
    expect(toInput(cards).map((s) => [s.oid[0], s.kind])).toEqual([
      ["a", "pick"],
      ["b", "pick"],
      ["c", "pick"],
      ["d", "pick"],
      ["e", "pick"],
    ]);
  });

  it("옮기기와 경계", () => {
    const cards = fromPlan(plan());
    expect(order(move(cards, 0, -1))).toBe("edcba");
    expect(order(move(cards, 4, 1))).toBe("edcba");
    const moved = move(cards, 2, -1);
    expect(order(moved)).toBe("ecdba");
    expect(status(moved, cards, 1)).toBe("moved");
    expect(status(moved, cards, 4)).toBeNull();
    expect(changed(moved, cards)).toBe(true);
  });

  it("묶음은 한 덩어리로 움직이고, 풀면 따로", () => {
    let cards = fromPlan(plan());
    cards = joinWithBelow(cards, 1); // d를 c에
    expect(groups(cards)).toEqual([[0, 0], [1, 2], [3, 3], [4, 4]]);
    expect(status(cards, cards, 1)).toBe("joined");
    const down = move(cards, 1, 1);
    expect(order(down)).toBe("ebdca");
    const up = move(cards, 2, -1);
    expect(order(up)).toBe("dceba");
    const apart = split(cards, 1);
    expect(order(move(apart, 1, 1))).toBe("ecdba");
  });

  it("빠진 카드는 묶을 수 없고, 묶인 카드를 빼면 그 카드만 빠진다", () => {
    let cards = fromPlan(plan());
    cards = toggleDrop(cards, 2); // c를 뺌
    expect(canJoin(cards, 1)).toBe(false);
    expect(canJoin(cards, 2)).toBe(false);
    expect(joinWithBelow(cards, 1)).toEqual(cards);
    cards = toggleDrop(cards, 2);
    cards = joinWithBelow(joinWithBelow(cards, 0), 1); // e·d를 c에
    expect(groups(cards)[0]).toEqual([0, 2]);
    // 머리(c)를 빼면 위(d)가 새 머리가 되고 아래로 이어 붙지 않는다.
    const headDropped = toggleDrop(cards, 2);
    expect(headDropped[1]!.joinBelow).toBe(false);
    expect(headDropped[0]!.joinBelow).toBe(true);
    expect(groups(headDropped)).toEqual([[0, 1], [2, 2], [3, 3], [4, 4]]);
    // 묶인 가운데 카드(d)를 빼면 그 카드만 빠지고 e는 d가 아닌… d에 이어 붙던 끈이 풀린다.
    const middle = toggleDrop(cards, 1);
    expect(middle[1]!.state).toBe("drop");
    expect(middle[0]!.joinBelow).toBe(false);
    expect(summary(middle)).toEqual({ after: 4, dropped: 1, joined: 0 });
  });

  it("메시지는 묶음의 머리에만, 처음 메시지와 같으면 고치지 않은 것", () => {
    let cards = joinWithBelow(fromPlan(plan()), 1);
    expect(setMessage(cards, 1, "x")).toEqual(cards);
    cards = setMessage(cards, 2, "c와 d\n\n본문");
    expect(cards[2]!.newMessage).toBe("c와 d\n\n본문");
    expect(setMessage(cards, 2, "c")[2]!.newMessage).toBeNull();
    // 묶이면 메시지는 머리 것만 남는다.
    const withMessage = setMessage(fromPlan(plan()), 1, "새 d");
    expect(joinWithBelow(withMessage, 1)[1]!.newMessage).toBeNull();
  });

  it("core 입력은 아래부터, 바뀐 메시지만", () => {
    let cards = fromPlan(plan());
    cards = joinWithBelow(cards, 1);
    cards = setMessage(cards, 2, "c와 d");
    cards = toggleDrop(cards, 4);
    cards = move(cards, 0, 1);
    expect(order(cards)).toBe("dceba");
    expect(toInput(cards)).toEqual([
      { oid: "a".repeat(40), kind: "drop", message: null },
      { oid: "b".repeat(40), kind: "pick", message: null },
      { oid: "e".repeat(40), kind: "pick", message: null },
      { oid: "c".repeat(40), kind: "pick", message: "c와 d" },
      { oid: "d".repeat(40), kind: "squash", message: null },
    ]);
    expect(summary(cards)).toEqual({ after: 3, dropped: 1, joined: 1 });
  });

  it("많은 카드에서도 맨 아래 묶음이 옮겨진다", () => {
    const cards = fromPlan(plan(26));
    expect(groups(cards)).toHaveLength(26);
    expect(order(move(cards, 25, -1)).slice(-2)).toBe("ab");
  });
});
