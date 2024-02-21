/*
struct Foo {
  x: int?,
  bar: [[Bar]]
}

struct Bar {
  y: int
}
*/

/*
{
  "x": null,
  "bar": [
    [
      {
        "y": 71
      },
      {
        "y": 72
      }
    ],
    [
      {
        "y": 41
      },
      {
        "y": 42
      }
    ]
  ]
}
*/

export type Constructor<T> = new (...arguments_: any) => T;
export type Instance<I> = I extends Constructor<infer T> ? T : never;

let _messageMap: Map<string, Constructor<StructMessage>> = new Map();
let _fieldsMap: Map<Constructor<StructMessage>, StructField[]> = new Map();

abstract class StructMessage {}

const enum TyKindTag {
  Primitive,
  Message,
  Array,
}

type MessageClassID = string | Constructor<StructMessage>;

type TyKind =
  | { kind: TyKindTag.Primitive }
  | { kind: TyKindTag.Message; of: MessageClassID }
  | { kind: TyKindTag.Array; of: TyKind };

type StructField = {
  name: string;
  ty: TyKind;
};

function valueToPlainObject(value: any, ty: TyKind) {
  if (value === null) {
    return null;
  }
  //
  else if (ty.kind === TyKindTag.Primitive) {
    return value;
  }
  //
  else if (ty.kind === TyKindTag.Message) {
    let result: Record<string, any> = {};

    let fields = _fieldsMap.get(Object.getPrototypeOf(value).constructor)!;
    for (let i = 0; i < fields.length; ++i) {
      const { name: fieldName, ty } = fields[i];
      const fieldVal = value[fieldName];
      result[fieldName] = valueToPlainObject(fieldVal, ty);
    }

    return result;
  }
  //
  else if (ty.kind === TyKindTag.Array) {
    return (value as any[]).map((val) => valueToPlainObject(val, ty.of));
  } else {
    throw new Error("Invalid value/ty");
  }
}

function plainObjectToValue(obj: any, ty: TyKind) {
  if (obj === null) {
    return null;
  }
  //
  else if (ty.kind === TyKindTag.Primitive) {
    return obj;
  } else if (ty.kind === TyKindTag.Message) {
    let ctor = typeof ty.of === "string" ? _messageMap.get(ty.of)! : ty.of;
    let fields = _fieldsMap.get(ctor)!;

    let createPayload = {};
    for (let i = 0; i < fields.length; ++i) {
      const { name: fieldName, ty } = fields[i];
      const fieldVal = obj[fieldName];
      createPayload[fieldName] = plainObjectToValue(fieldVal, ty);
    }

    // TODO: Make this strongly typed
    return (ctor as any).create(createPayload);
  }
  //
  else if (ty.kind === TyKindTag.Array) {
    return (obj as any[]).map((val) => plainObjectToValue(val, ty.of));
  }
  //
  else {
    throw new Error("Invalid value/ty");
  }
}

namespace forser {
  export function packMessage<M extends StructMessage>(message: M) {
    return JSON.stringify(
      valueToPlainObject(message, {
        kind: TyKindTag.Message,
        of: Object.getPrototypeOf(message).constructor,
      })
    );
  }

  export function unpackMessage<M extends Constructor<StructMessage>>(
    messageType: M,
    serialized: string
  ): Instance<M> {
    let obj = JSON.parse(serialized);
    return plainObjectToValue(obj, {
      kind: TyKindTag.Message,
      of: messageType,
    });
  }
}

type FieldsOf<T> = Pick<
  T,
  {
    [K in keyof T]: T[K] extends Function ? never : K;
  }[keyof T]
>;

/* =========================================== */

const _FooFields: StructField[] = [
  { name: "x", ty: { kind: TyKindTag.Primitive } },
  {
    name: "bar",
    ty: {
      kind: TyKindTag.Array,
      of: { kind: TyKindTag.Array, of: { kind: TyKindTag.Message, of: "Bar" } },
    },
  },
];

export class Foo extends StructMessage {
  static create(body: FieldsOf<Foo>): Foo {
    return Object.assign(new Foo(), body);
  }

  public x!: number | null;
  public bar!: Bar[][];
}

_messageMap.set("Foo", Foo);
_fieldsMap.set(Foo, _FooFields);

/* =========================================== */

const _BarFields: StructField[] = [
  { name: "y", ty: { kind: TyKindTag.Primitive } },
];

export class Bar extends StructMessage {
  static create(body: FieldsOf<Bar>): Bar {
    return Object.assign(new Bar(), body);
  }

  public y!: number;
}

_messageMap.set("Bar", Bar);
_fieldsMap.set(Bar, _BarFields);

/* =========================================== */


let foo1 = Foo.create({
  x: null,
  bar: [
    [Bar.create({ y: 71 }), Bar.create({ y: 72 })],
    [Bar.create({ y: 41 }), Bar.create({ y: 42 })],
  ],
});

console.log(Object.getPrototypeOf(foo1.bar[0][0]).constructor);

let json = `{
  "x": null,
  "bar": [
    [
      {
        "y": 71
      },
      {
        "y": 72
      }
    ],
    [
      {
        "y": 41
      },
      {
        "y": 42
      }
    ]
  ]
}`;

let foo2 = forser.unpackMessage(Foo, json);
console.log(Object.getPrototypeOf(foo2.bar[0][0]).constructor);
